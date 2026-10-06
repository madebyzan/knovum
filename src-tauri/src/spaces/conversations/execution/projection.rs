use serde_json::{json, Value};
use sqlx::SqlitePool;

use crate::{
    agent_runtime::protocol::messages::AgentProjection,
    spaces::{
        conversations::{builtin_agent::identity::TaskIds, repository},
        db,
    },
};

pub(in crate::spaces::conversations) async fn commit(
    pool: &SqlitePool,
    ids: &TaskIds,
    projection: AgentProjection,
) -> Result<Value, String> {
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let state: (String, Option<i64>) = sqlx::query_as("SELECT segment_id,r.cancel_requested_at FROM spaces_builtin_agent_tasks t JOIN spaces_runs r ON r.id=t.run_id WHERE t.id=$1")
        .bind(&ids.task_id).fetch_one(&mut *tx).await.map_err(db::sql_error)?;
    if state.0 != projection.thread_id {
        return Err("builtin-agent-projection-thread-mismatch".into());
    }
    let plan = json!({"items":projection.plan});
    let previous_plan: Option<String> = sqlx::query_scalar("SELECT payload FROM spaces_run_events WHERE run_id=$1 AND kind='builtin_agent_plan_updated' ORDER BY sequence DESC LIMIT 1")
        .bind(&ids.run_id).fetch_optional(&mut *tx).await.map_err(db::sql_error)?;
    let changed = previous_plan
        .as_deref()
        .map(serde_json::from_str::<Value>)
        .transpose()
        .map_err(db::json_error)?
        .as_ref()
        != Some(&plan);
    if changed && (!projection.plan.is_empty() || previous_plan.is_some()) {
        crate::spaces::runs::append_event(
            &mut tx,
            &ids.run_id,
            None,
            None,
            crate::spaces::runs::RunEventKind::BuiltinAgentPlanUpdated,
            &plan,
            now,
        )
        .await?;
    }
    for message in projection.messages {
        let previous: Option<String> = sqlx::query_scalar("SELECT message_id FROM spaces_conversation_message_projections WHERE segment_id=$1 AND provider_message_id=$2")
            .bind(&projection.thread_id).bind(&message.message_id).fetch_optional(&mut *tx).await.map_err(db::sql_error)?;
        if previous.is_some() {
            continue;
        }
        let id = repository::append_message_in_transaction(&mut tx, &ids.conversation_id, &ids.branch_id, repository::ConversationMessageRole::Assistant,
            &json!({"parts":[{"type":"text","text":message.text}],"runId":ids.run_id,"status":state.1.is_some().then_some("stopped")}), now).await?;
        sqlx::query("INSERT INTO spaces_conversation_message_projections(segment_id,provider_message_id,message_id) VALUES($1,$2,$3)")
            .bind(&projection.thread_id).bind(message.message_id).bind(id).execute(&mut *tx).await.map_err(db::sql_error)?;
    }
    for input_id in projection.accepted_input_ids {
        sqlx::query("UPDATE spaces_builtin_agent_tasks SET input_accepted=1 WHERE segment_id=$1 AND input_id=$2")
            .bind(&projection.thread_id).bind(input_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    }
    for checkpoint in projection.model_checkpoints {
        for request_id in checkpoint.request_ids {
            sqlx::query("UPDATE spaces_builtin_agent_model_calls SET state='committed' WHERE id=$1 AND graph_namespace=$2 AND state='received' AND task_id IN (SELECT id FROM spaces_builtin_agent_tasks WHERE segment_id=$3)")
                .bind(request_id).bind(&checkpoint.graph_namespace).bind(&projection.thread_id).execute(&mut *tx).await.map_err(db::sql_error)?;
        }
        for message_id in checkpoint.message_ids {
            sqlx::query("UPDATE spaces_builtin_agent_model_calls SET state='committed' WHERE message_id=$1 AND graph_namespace=$2 AND state='received' AND task_id IN (SELECT id FROM spaces_builtin_agent_tasks WHERE segment_id=$3)")
                .bind(message_id).bind(&checkpoint.graph_namespace).bind(&projection.thread_id).execute(&mut *tx).await.map_err(db::sql_error)?;
        }
    }
    sqlx::query(
        "UPDATE spaces_builtin_agent_execution_segments SET projection_checkpoint_id=$1 WHERE id=$2",
    )
    .bind(projection.checkpoint_id)
    .bind(&projection.thread_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(Value::Null)
}
