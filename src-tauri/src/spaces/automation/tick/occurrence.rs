use sqlx::{Row, SqlitePool};

use crate::spaces::{db, workflows};

use super::super::{contract::OverlapPolicy, repository, AutomationTrigger};

pub(super) async fn reserve(
    pool: &SqlitePool,
    trigger_id: &str,
    scheduled_at: i64,
    state: &str,
    now: i64,
) -> Result<(), String> {
    sqlx::query(
        "INSERT OR IGNORE INTO spaces_automation_occurrences(
         id,trigger_id,scheduled_at,occurrence_key,run_id,state,detail,created_at)
         VALUES($1,$2,$3,$4,NULL,$5,'{}',$6)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(trigger_id)
    .bind(scheduled_at)
    .bind(scheduled_at.to_string())
    .bind(state)
    .bind(now)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

pub(super) async fn drain(pool: &SqlitePool, now: i64) -> Result<(), String> {
    let rows = sqlx::query(
        "SELECT o.id,o.trigger_id,o.detail FROM spaces_automation_occurrences o \
         JOIN spaces_automation_triggers t ON t.id=o.trigger_id \
         WHERE o.state='queued' AND t.enabled=1 ORDER BY o.scheduled_at LIMIT 20",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    for row in rows {
        let occurrence_id: String = row.try_get("id").map_err(db::sql_error)?;
        let trigger_id: String = row.try_get("trigger_id").map_err(db::sql_error)?;
        let detail: String = row.try_get("detail").map_err(db::sql_error)?;
        if let Err(error) = start(pool, &occurrence_id, &trigger_id, &detail, now).await {
            mark(pool, &occurrence_id, "failed", Some(&error), None).await?;
        }
    }
    Ok(())
}

pub(in crate::spaces::automation) async fn test(
    pool: &SqlitePool,
    trigger_id: &str,
) -> Result<String, String> {
    let now = crate::clock::unix_time_millis()?;
    let occurrence_id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO spaces_automation_occurrences(
         id,trigger_id,scheduled_at,occurrence_key,run_id,state,detail,created_at)
         VALUES($1,$2,$3,$4,NULL,'queued',$5,$3)",
    )
    .bind(&occurrence_id)
    .bind(trigger_id)
    .bind(now)
    .bind(format!("test:{occurrence_id}"))
    .bind(r#"{"source":"manual_test"}"#)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    start(
        pool,
        &occurrence_id,
        trigger_id,
        r#"{"source":"manual_test"}"#,
        now,
    )
    .await?;
    sqlx::query_scalar("SELECT run_id FROM spaces_automation_occurrences WHERE id=$1")
        .bind(&occurrence_id)
        .fetch_one(pool)
        .await
        .map_err(db::sql_error)
}

async fn start(
    pool: &SqlitePool,
    occurrence_id: &str,
    trigger_id: &str,
    encoded_detail: &str,
    now: i64,
) -> Result<(), String> {
    let trigger = repository::load(pool, trigger_id).await?;
    if trigger.valid_until < now {
        return mark(pool, occurrence_id, "authorization_expired", None, None).await;
    }
    if has_active(pool, trigger_id).await? {
        match trigger.overlap_policy {
            OverlapPolicy::Skip => {
                return mark(pool, occurrence_id, "skipped_overlap", None, None).await
            }
            OverlapPolicy::Queue => return Ok(()),
            OverlapPolicy::Parallel => {}
            OverlapPolicy::CancelPrevious => cancel_active(pool, trigger_id, now).await?,
        }
    }
    let detail = serde_json::from_str(encoded_detail).map_err(db::json_error)?;
    let run =
        workflows::start_with_run_id(pool, workflow_request(trigger, detail), Some(occurrence_id))
            .await?;
    mark(pool, occurrence_id, "started", None, Some(&run.id)).await
}

fn workflow_request(
    trigger: AutomationTrigger,
    detail: serde_json::Value,
) -> workflows::StartWorkflowRequest {
    let input = serde_json::json!({
        "configured": trigger.input,
        "automationEvent": detail,
    });
    workflows::StartWorkflowRequest {
        space_id: trigger.space_id,
        workflow_id: trigger.workflow_id,
        version: trigger.workflow_version,
        input,
        parent_run_id: None,
        grant_id: Some(trigger.grant_id),
        test_mode: false,
        breakpoint_node_ids: Vec::new(),
    }
}

async fn has_active(pool: &SqlitePool, trigger_id: &str) -> Result<bool, String> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_automation_occurrences o JOIN spaces_runs r ON r.id=o.run_id \
         WHERE o.trigger_id=$1 AND r.completed_at IS NULL",
    )
    .bind(trigger_id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(count > 0)
}

async fn cancel_active(pool: &SqlitePool, trigger_id: &str, now: i64) -> Result<(), String> {
    sqlx::query(
        "UPDATE spaces_runs SET cancel_requested_at=COALESCE(cancel_requested_at,$1), \
         execution_epoch=execution_epoch+CASE WHEN cancel_requested_at IS NULL THEN 1 ELSE 0 END,updated_at=$1 \
         WHERE completed_at IS NULL AND id IN (SELECT run_id FROM spaces_automation_occurrences \
         WHERE trigger_id=$2 AND run_id IS NOT NULL)",
    ).bind(now).bind(trigger_id).execute(pool).await.map_err(db::sql_error)?;
    Ok(())
}

async fn mark(
    pool: &SqlitePool,
    id: &str,
    state: &str,
    detail: Option<&str>,
    run_id: Option<&str>,
) -> Result<(), String> {
    let detail = serde_json::json!({"message": detail});
    sqlx::query("UPDATE spaces_automation_occurrences SET state=$1,detail=$2,run_id=$3 WHERE id=$4 AND state='queued'")
        .bind(state).bind(serde_json::to_string(&detail).map_err(db::json_error)?)
        .bind(run_id).bind(id).execute(pool).await.map_err(db::sql_error)?;
    Ok(())
}
