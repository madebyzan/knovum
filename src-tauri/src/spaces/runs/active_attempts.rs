use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use tokio_util::sync::CancellationToken;

use super::AttemptClaim;

struct ActiveAttempt {
    run_id: String,
    cancellation: CancellationToken,
}

fn registry() -> &'static Mutex<HashMap<String, ActiveAttempt>> {
    static REGISTRY: OnceLock<Mutex<HashMap<String, ActiveAttempt>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(super) struct Registration(String);

impl Drop for Registration {
    fn drop(&mut self) {
        match registry().lock() {
            Ok(mut active) => {
                active.remove(&self.0);
            }
            Err(error) => log::error!("Run attempt registry is unavailable: {error}"),
        }
    }
}

pub(super) fn register(claim: &AttemptClaim) -> Result<Registration, String> {
    let mut active = registry().lock().map_err(|error| error.to_string())?;
    if active.contains_key(&claim.attempt_id) {
        return Err("Run attempt is already executing locally".into());
    }
    active.insert(
        claim.attempt_id.clone(),
        ActiveAttempt {
            run_id: claim.run_id.clone(),
            cancellation: claim.cancellation.clone(),
        },
    );
    Ok(Registration(claim.attempt_id.clone()))
}

pub(super) async fn cancel_tree(pool: &sqlx::SqlitePool, run_id: &str) -> Result<(), String> {
    let runs: Vec<String> = sqlx::query_scalar(
        "WITH RECURSIVE run_tree(id) AS (
         SELECT id FROM spaces_runs WHERE id=?
         UNION SELECT child.id FROM spaces_runs child JOIN run_tree parent ON child.parent_run_id=parent.id)
         SELECT id FROM run_tree",
    ).bind(run_id).fetch_all(pool).await.map_err(crate::spaces::db::sql_error)?;
    let active = registry().lock().map_err(|error| error.to_string())?;
    for attempt in active
        .values()
        .filter(|attempt| runs.contains(&attempt.run_id))
    {
        attempt.cancellation.cancel();
    }
    Ok(())
}

// SQL owns validity. These tokens only interrupt work after cancellation, expiry or takeover.
pub(super) async fn reconcile(pool: &sqlx::SqlitePool) -> Result<(), String> {
    let attempts: Vec<(String, CancellationToken)> = registry()
        .lock()
        .map_err(|error| error.to_string())?
        .iter()
        .map(|(id, attempt)| (id.clone(), attempt.cancellation.clone()))
        .collect();
    if attempts.is_empty() {
        return Ok(());
    }
    let ids: Vec<&str> = attempts.iter().map(|(id, _)| id.as_str()).collect();
    let valid: Vec<String> = sqlx::query_scalar(
        "SELECT a.id FROM spaces_run_attempts a JOIN spaces_runs r ON r.id=a.run_id
         JOIN spaces_run_steps s ON s.id=a.step_instance_id AND s.run_id=a.run_id
         WHERE a.id IN (SELECT value FROM json_each(?)) AND a.state IN ('claimed','executing')
         AND a.execution_epoch=r.execution_epoch AND a.lease_until>=?
         AND r.cancel_requested_at IS NULL AND r.completed_at IS NULL AND s.state='running'",
    )
    .bind(serde_json::to_string(&ids).map_err(crate::spaces::db::json_error)?)
    .bind(crate::clock::unix_time_millis()?)
    .fetch_all(pool)
    .await
    .map_err(crate::spaces::db::sql_error)?;
    for (id, token) in attempts {
        if !valid.contains(&id) {
            token.cancel();
        }
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/active_attempts.rs"]
mod tests;
