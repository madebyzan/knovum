use sqlx::{Row, SqlitePool};

use super::ReconciliationDecision;
use crate::spaces::{db, runs::RunEventKind};

pub(crate) async fn reconcile(
    pool: &SqlitePool,
    step_id: &str,
    expected_version: i64,
    decision: ReconciliationDecision,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let row = sqlx::query(
        "SELECT s.run_id FROM spaces_run_steps s JOIN spaces_runs r ON r.id=s.run_id
         WHERE s.id=$1 AND s.state='blocked' AND s.wait_reason='external_reconciliation'
         AND s.state_version=$2 AND r.cancel_requested_at IS NULL
         AND r.pause_requested_at IS NULL AND r.completed_at IS NULL",
    )
    .bind(step_id)
    .bind(expected_version)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("run-step-stale: reconciliation checkpoint changed")?;
    let run_id: String = row.try_get("run_id").map_err(db::sql_error)?;
    let state = match decision {
        ReconciliationDecision::Retry => "ready",
        ReconciliationDecision::Fail => "failed",
    };
    sqlx::query(
        "UPDATE spaces_run_steps SET state=$1,wait_reason=NULL,state_version=state_version+1,
         available_at=$2,updated_at=$2 WHERE id=$3",
    )
    .bind(state)
    .bind(now)
    .bind(step_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if matches!(decision, ReconciliationDecision::Fail) {
        sqlx::query("UPDATE spaces_run_steps SET state='blocked',wait_reason=NULL,state_version=state_version+1,
             updated_at=$1 WHERE run_id=$2 AND id<>$3 AND state IN ('pending','ready','waiting','paused')")
            .bind(now).bind(&run_id).bind(step_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    }
    let run_state = match decision {
        ReconciliationDecision::Retry => "queued",
        ReconciliationDecision::Fail => "failed",
    };
    let completed_at = matches!(decision, ReconciliationDecision::Fail).then_some(now);
    let run = sqlx::query(
        "UPDATE spaces_runs SET state=$1,wait_reason=NULL,state_version=state_version+1,
         updated_at=$2,completed_at=$3 WHERE id=$4 AND cancel_requested_at IS NULL
         AND pause_requested_at IS NULL AND completed_at IS NULL",
    )
    .bind(run_state)
    .bind(now)
    .bind(completed_at)
    .bind(&run_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(run, "run-step-stale: reconciliation owner changed")?;
    super::super::append_event(
        &mut tx,
        &run_id,
        Some(step_id),
        None,
        RunEventKind::ExternalOutcomeReconciled,
        &serde_json::json!({"decision": state}),
        now,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    super::super::signals::announce();
    Ok(())
}

#[cfg(test)]
mod tests {
    use sqlx::sqlite::SqlitePoolOptions;

    use super::{reconcile, ReconciliationDecision};

    #[tokio::test]
    async fn cancelled_run_rejects_stale_reconciliation_without_changing_step_or_epoch() {
        let pool = fixture().await;
        crate::spaces::runs::repository::request_control(&pool, "run", "cancel")
            .await
            .unwrap();

        let error = reconcile(&pool, "step", 3, ReconciliationDecision::Fail)
            .await
            .unwrap_err();
        assert_eq!(error, "run-step-stale: reconciliation checkpoint changed");
        let row: (String, i64, Option<i64>, Option<i64>, String, i64) = sqlx::query_as(
            "SELECT r.state,r.execution_epoch,r.cancel_requested_at,r.completed_at,
             s.state,s.state_version FROM spaces_runs r JOIN spaces_run_steps s ON s.run_id=r.id
             WHERE r.id='run'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row.0, "waiting");
        assert_eq!(row.1, 5);
        assert!(row.2.is_some());
        assert!(row.3.is_none());
        assert_eq!(row.4, "blocked");
        assert_eq!(row.5, 3);
    }

    #[tokio::test]
    async fn successful_reconciliation_announces_its_committed_transition() {
        let pool = fixture().await;
        let mut changes = crate::spaces::runs::subscribe_to_changes();
        let version = *changes.borrow_and_update();

        reconcile(&pool, "step", 3, ReconciliationDecision::Retry)
            .await
            .unwrap();

        assert_ne!(*changes.borrow_and_update(), version);
        let row: (String, String) = sqlx::query_as(
            "SELECT r.state,s.state FROM spaces_runs r JOIN spaces_run_steps s ON s.run_id=r.id
             WHERE r.id='run'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(row, ("queued".into(), "ready".into()));
    }

    async fn fixture() -> sqlx::SqlitePool {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();
        crate::spaces::db::migrate(&pool).await.unwrap();
        sqlx::query("INSERT INTO spaces(id,kind,title,created_at,updated_at) VALUES('space','space','Space',1,1)")
            .execute(&pool).await.unwrap();
        sqlx::query(
            "INSERT INTO spaces_runs(id,home_space_id,principal_kind,principal_id,kind,state,
             execution_epoch,wait_reason,policy_snapshot,input_snapshot,budget,created_at,updated_at)
             VALUES('run','space','assistant','principal','action','waiting',4,
             'external_reconciliation','{}','{}','{}',1,1)",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,state,
             state_version,resolved_input,wait_reason,created_at,updated_at)
             VALUES('step','run','node','node','action','blocked',3,'{}',
             'external_reconciliation',1,1)",
        )
        .execute(&pool)
        .await
        .unwrap();
        pool
    }
}
