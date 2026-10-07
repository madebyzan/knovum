use sqlx::SqlitePool;

use crate::{
    agent_runtime::{process::Peer, protocol::control::AgentPruneThreads},
    spaces::db,
};

pub(in crate::spaces::conversations) async fn prune(
    pool: &SqlitePool,
    peer: &Peer,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let cutoff = now - 30 * 24 * 60 * 60 * 1_000;
    let threads: Vec<String> = sqlx::query_scalar("SELECT s.id FROM spaces_builtin_agent_execution_segments s
        WHERE s.checkpoint_deleted_at IS NULL AND s.projection_checkpoint_id IS NOT NULL
        AND s.id!=(SELECT latest.id FROM spaces_builtin_agent_execution_segments latest WHERE latest.conversation_id=s.conversation_id AND latest.branch_id=s.branch_id ORDER BY latest.created_at DESC,latest.rowid DESC LIMIT 1)
        AND NOT EXISTS(SELECT 1 FROM spaces_builtin_agent_tasks t JOIN spaces_runs r ON r.id=t.run_id WHERE t.segment_id=s.id AND (r.completed_at IS NULL OR r.updated_at>=$1 OR t.input_accepted=0))
        AND NOT EXISTS(SELECT 1 FROM spaces_builtin_agent_model_calls c JOIN spaces_builtin_agent_tasks t ON t.id=c.task_id WHERE t.segment_id=s.id AND c.state!='committed' AND c.retry_authorized_at IS NULL)
        ORDER BY s.created_at LIMIT 100")
        .bind(cutoff).fetch_all(pool).await.map_err(db::sql_error)?;
    let retained_thread_ids =
        sqlx::query_scalar("SELECT id FROM spaces_builtin_agent_execution_segments")
            .fetch_all(pool)
            .await
            .map_err(db::sql_error)?;
    peer.request::<serde_json::Value>(
        "agentRuntime.pruneThreads",
        &AgentPruneThreads {
            thread_ids: threads.clone(),
            retained_thread_ids,
            orphan_before_ms: cutoff,
        },
    )
    .await?;
    if threads.is_empty() {
        return Ok(());
    }
    let mut tx = db::begin_write(pool).await?;
    sqlx::query(
        "UPDATE spaces_builtin_agent_execution_segments SET checkpoint_deleted_at=$1 \
         WHERE id IN (SELECT value FROM json_each($2))",
    )
    .bind(now)
    .bind(serde_json::to_string(&threads).map_err(db::json_error)?)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(())
}
