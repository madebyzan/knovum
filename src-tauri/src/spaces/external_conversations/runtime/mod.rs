use std::sync::Arc;

use tauri::AppHandle;

use super::{repository, StartExternalConversationRequest};
use crate::{extensions::contracts::AgentExtensionSnapshot, spaces::db};

pub(super) mod lifecycle;
pub(super) mod pool;

use lifecycle::mark_disconnected;

/// One user message handed to a desktop Agent: its records, the runtime it runs on, and where
/// its updates go. Dropping it releases the runtime for the conversation's next turn.
pub struct AgentTurn {
    pub binding: super::ExternalAgentSessionBinding,
    lease: pool::Lease,
    on_update: crate::external_agents::session::AgentUpdateHandler,
}

impl AgentTurn {
    /// The Agent already holds the conversation, so the prompt carries only the new request.
    pub fn continues(&self) -> bool {
        self.lease.continues
    }
}

pub async fn start(
    app: AppHandle,
    request: StartExternalConversationRequest,
    conversation: Option<(String, String)>,
    updates: tauri::ipc::Channel<crate::spaces::conversations::ConversationTaskUpdate>,
) -> Result<AgentTurn, String> {
    lifecycle::validate(&request)?;
    crate::ai::settings::ensure_agent_enabled(&app, request.runtime_kind)
        .map_err(|error| error.to_string())?;
    let extensions: Arc<AgentExtensionSnapshot> = Arc::new(
        serde_json::from_value(request.input_snapshot["extensions"].clone())
            .map_err(db::json_error)?,
    );
    let id = uuid::Uuid::new_v4().to_string();
    let run_id = uuid::Uuid::new_v4().to_string();
    let (conversation_id, branch_id) = conversation
        .clone()
        .unwrap_or_else(|| (id.clone(), id.clone()));
    let pool = db::pool(&app).await?;
    let head = repository::branch_head(pool, &conversation_id, &branch_id).await?;
    lifecycle::create::create_records(
        &app,
        &request,
        &id,
        &run_id,
        conversation
            .as_ref()
            .map(|(id, branch)| (id.as_str(), branch.as_str())),
    )
    .await?;
    let wanted = pool::Wanted {
        space_id: &request.space_id,
        conversation_id: &conversation_id,
        branch_id: &branch_id,
        kind: request.runtime_kind,
        head: head.as_deref(),
        extensions: &extensions,
    };
    let lease = match pool::acquire(&app, wanted, &id).await {
        Ok(lease) => lease,
        Err(error) => {
            mark_disconnected(pool, &id, &run_id, &error).await?;
            return Err(error);
        }
    };
    let connected = async {
        lease.authorize(&app, &id, &run_id, extensions).await?;
        let on_update = super::updates::callback(pool, id.clone(), run_id.clone(), Some(updates))?;
        if let Err(error) = lifecycle::mark_connected(pool, &id, &run_id, lease.session()).await {
            super::updates::release(&id);
            return Err(error);
        }
        Ok::<_, String>(on_update)
    }
    .await;
    match connected {
        Ok(on_update) => Ok(AgentTurn {
            binding: lifecycle::load(pool, &id).await?,
            lease,
            on_update,
        }),
        Err(error) => {
            mark_disconnected(pool, &id, &run_id, &error).await?;
            Err(error)
        }
    }
}

pub async fn begin_turn(
    app: AppHandle,
    conversation_id: &str,
    prompt: &str,
    attachments: &[crate::spaces::conversations::ConversationMessageAttachment],
) -> Result<String, String> {
    repository::start_turn(db::pool(&app).await?, conversation_id, prompt, attachments).await
}

/// Sends the turn's prompt and settles the turn once the Agent has answered, stopped, or run out
/// of the Run's time budget. The runtime outlives the turn whatever its outcome.
pub async fn send_prepared(
    app: AppHandle,
    turn: &mut AgentTurn,
    turn_id: &str,
    prepared: crate::external_agents::prompt::AgentPrompt,
) -> Result<(), String> {
    let pool = db::pool(&app).await?;
    let binding_id = turn.binding.id.clone();
    let flush = super::updates::flush(&binding_id);
    let outcome = within_budget(pool, turn, prepared).await;
    turn.lease.settle(
        repository::branch_head(pool, &turn.binding.conversation_id, &turn.binding.branch_id)
            .await?,
    );
    if let Err(error) = flush.await {
        repository::finish_turn(pool, turn_id, "failed", Some(&error)).await?;
        return Err(error);
    }
    match outcome {
        Ok(()) => {
            if !repository::has_turn_output(pool, turn_id).await?
                && crate::external_agents::artifacts::collect(turn.lease.workspace())
                    .map_err(|error| error.to_string())?
                    .is_empty()
            {
                let error =
                    "Desktop Agent completed without a reply. Check its model configuration and retry.";
                repository::finish_turn(pool, turn_id, "failed", Some(error)).await?;
                return Err(error.into());
            }
            repository::finish_turn(pool, turn_id, "completed", None).await
        }
        Err(error) => {
            let message = error.to_string();
            let state = if error.code == "cancelled" {
                "cancelled"
            } else {
                "failed"
            };
            repository::finish_turn(pool, turn_id, state, Some(&message)).await?;
            Err(message)
        }
    }
}

/// Runs the prompt until it ends or the Run's duration budget is spent; a spent budget stops
/// the Agent's turn like a cancellation does, then reports why.
async fn within_budget(
    pool: &sqlx::SqlitePool,
    turn: &AgentTurn,
    prepared: crate::external_agents::prompt::AgentPrompt,
) -> crate::foundation::app_error::AppResult<()> {
    let deadline_at: i64 = sqlx::query_scalar(
        "SELECT created_at + CAST(json_extract(budget,'$.maxDurationSeconds') AS INTEGER)*1000 \
         FROM spaces_runs WHERE id=$1",
    )
    .bind(&turn.binding.run_id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    let remaining = deadline_at.saturating_sub(crate::foundation::clock::unix_time_millis()?);
    let cancellation = turn.lease.cancellation();
    let send = crate::external_agents::conversation::send(
        turn.lease.runtime_id(),
        prepared,
        turn.on_update.clone(),
        cancellation.clone(),
    );
    tokio::pin!(send);
    tokio::select! {
        outcome = &mut send => outcome,
        _ = tokio::time::sleep(std::time::Duration::from_millis(remaining.max(0) as u64)) => {
            cancellation.cancel();
            let _stopped = send.await;
            Err(crate::foundation::app_error::AppError::new(
                "agent_turn_budget_exhausted",
                "External Agent turn exceeded its duration budget",
            ))
        }
    }
}
