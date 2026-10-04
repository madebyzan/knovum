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
mod tests {
    use super::*;

    async fn claim(pool: &sqlx::SqlitePool, parent: Option<&str>) -> AttemptClaim {
        let run_id = uuid::Uuid::new_v4().to_string();
        let step_instance_id = uuid::Uuid::new_v4().to_string();
        let attempt_id = uuid::Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO spaces_runs(id,home_space_id,parent_run_id,principal_kind,principal_id,kind,state,policy_snapshot,input_snapshot,budget,created_at,updated_at) VALUES(?,'space',?,'user','user','workflow','running','{}','{}','{}',1,1)")
            .bind(&run_id).bind(parent).execute(pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,state,resolved_input,created_at,updated_at) VALUES(?,?,'segment','segment','speech_segment','running','{}',1,1)")
            .bind(&step_instance_id).bind(&run_id).execute(pool).await.unwrap();
        sqlx::query("INSERT INTO spaces_run_attempts(id,run_id,step_instance_id,attempt_number,state,request_digest,executor_id,idempotency_key,execution_epoch,lease_until,started_at) VALUES(?,?,?,1,'executing',?,'desktop',?,0,?,1)")
            .bind(&attempt_id).bind(&run_id).bind(&step_instance_id).bind("a".repeat(64))
            .bind(&attempt_id).bind(crate::clock::unix_time_millis().unwrap()+60_000)
            .execute(pool).await.unwrap();
        AttemptClaim {
            attempt_id,
            run_id,
            step_instance_id,
            execution_epoch: 0,
            attempt_number: 1,
            home_space_id: "space".into(),
            node_kind: "speech_segment".into(),
            configuration: serde_json::json!({}),
            resolved_input: serde_json::json!({}),
            policy_snapshot: serde_json::json!({}),
            budget: serde_json::json!({}),
            cancellation: CancellationToken::new(),
        }
    }

    #[tokio::test]
    async fn cancellation_interrupts_descendants_and_registration_races_without_waiting_for_renewal(
    ) {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        crate::spaces::db::migrate(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
            .execute(&pool).await.unwrap();
        let root = claim(&pool, None).await;
        let child = claim(&pool, Some(&root.run_id)).await;
        let unrelated = claim(&pool, None).await;
        let _root = register(&root).unwrap();
        let _child = register(&child).unwrap();
        let _unrelated = register(&unrelated).unwrap();
        super::super::request_cancellation(&pool, &root.run_id)
            .await
            .unwrap();
        assert!(root.cancellation.is_cancelled());
        assert!(child.cancellation.is_cancelled());
        assert!(!unrelated.cancellation.is_cancelled());

        // Cancellation can commit between claiming and registering the local worker.
        let late = claim(&pool, None).await;
        super::super::request_cancellation(&pool, &late.run_id)
            .await
            .unwrap();
        let _late = register(&late).unwrap();
        reconcile(&pool).await.unwrap();
        assert!(late.cancellation.is_cancelled());
        assert!(!unrelated.cancellation.is_cancelled());
        sqlx::query("UPDATE spaces_runs SET execution_epoch=execution_epoch+1 WHERE id=?")
            .bind(&unrelated.run_id)
            .execute(&pool)
            .await
            .unwrap();
        reconcile(&pool).await.unwrap();
        assert!(unrelated.cancellation.is_cancelled());
    }
}
