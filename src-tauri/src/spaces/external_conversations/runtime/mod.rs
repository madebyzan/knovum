use std::sync::Arc;

use tauri::AppHandle;

use super::{repository, StartExternalConversationRequest};
use crate::{ai::capabilities, spaces::db};

mod lifecycle;

pub(super) use lifecycle::mark_disconnected;

pub async fn start(
    app: AppHandle,
    request: StartExternalConversationRequest,
    conversation: Option<(String, String)>,
    updates: Option<tauri::ipc::Channel<crate::spaces::assistant::AssistantTaskUpdate>>,
) -> Result<super::AgentSessionBinding, String> {
    lifecycle::validate(&request)?;
    crate::ai::settings::ensure_agent_enabled(&app, request.runtime_kind)
        .map_err(|error| error.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    let run_id = uuid::Uuid::new_v4().to_string();
    let workspace = capabilities::agent_artifacts::workspace(&app, &request.space_id, &id)
        .map_err(|error| error.to_string())?;
    capabilities::agent_artifacts::prepare(&workspace).map_err(|error| error.to_string())?;
    if let Err(error) = lifecycle::create_records(
        &app,
        &request,
        &id,
        &run_id,
        conversation
            .as_ref()
            .map(|(id, branch)| (id.as_str(), branch.as_str())),
    )
    .await
    {
        capabilities::agent_artifacts::discard(&workspace).map_err(|reason| reason.to_string())?;
        return Err(error);
    }
    let binding = match crate::spaces::mcp::agent_server::start(
        app.clone(),
        request.space_id.clone(),
        id.clone(),
    )
    .await
    {
        Ok(binding) => Arc::new(binding),
        Err(error) => {
            mark_disconnected(db::pool(&app).await?, &id, &run_id, &error).await?;
            capabilities::agent_artifacts::discard(&workspace)
                .map_err(|reason| reason.to_string())?;
            return Err(error);
        }
    };
    let callback = match super::updates::callback(app.clone(), id.clone(), run_id.clone(), updates)
    {
        Ok(callback) => callback,
        Err(error) => {
            mark_disconnected(db::pool(&app).await?, &id, &run_id, &error).await?;
            capabilities::agent_artifacts::discard(&workspace)
                .map_err(|reason| reason.to_string())?;
            return Err(error);
        }
    };
    let keepalive: Arc<dyn Send + Sync> = binding.clone();
    let runtime_session = crate::external_agents::conversation::start(
        id.clone(),
        request.runtime_kind,
        workspace.clone(),
        callback,
        vec![binding.server()],
        keepalive,
    )
    .await
    .map_err(|error| error.to_string());
    match runtime_session {
        Ok(session) => {
            if let Err(error) =
                lifecycle::mark_connected(db::pool(&app).await?, &id, &run_id, &session).await
            {
                let _ = crate::external_agents::conversation::close(&id).await;
                mark_disconnected(db::pool(&app).await?, &id, &run_id, &error).await?;
                capabilities::agent_artifacts::discard(&workspace)
                    .map_err(|reason| reason.to_string())?;
                return Err(error);
            }
            lifecycle::schedule_expiration(
                app.clone(),
                id.clone(),
                run_id.clone(),
                request.max_duration_seconds,
            );
            lifecycle::load(db::pool(&app).await?, &id).await
        }
        Err(error) => {
            super::updates::release(&id);
            mark_disconnected(db::pool(&app).await?, &id, &run_id, &error).await?;
            capabilities::agent_artifacts::discard(&workspace)
                .map_err(|reason| reason.to_string())?;
            Err(error)
        }
    }
}

pub async fn begin_turn(
    app: AppHandle,
    conversation_id: &str,
    prompt: &str,
    attachments: &[crate::spaces::assistant::AssistantMessageAttachment],
) -> Result<String, String> {
    repository::start_turn(db::pool(&app).await?, conversation_id, prompt, attachments).await
}

pub async fn send_prepared(
    app: AppHandle,
    conversation_id: &str,
    turn_id: &str,
    prepared: crate::external_agents::prompt::AgentPrompt,
) -> Result<(), String> {
    let pool = db::pool(&app).await?;
    let outcome = crate::external_agents::conversation::send(conversation_id, prepared).await;
    if let Err(error) = super::updates::flush(conversation_id).await {
        repository::finish_turn(pool, turn_id, "failed", Some(&error)).await?;
        return Err(error);
    }
    match outcome {
        Ok(()) => repository::finish_turn(pool, turn_id, "completed", None).await,
        Err(error) => {
            let message = error.to_string();
            let state = if error.code == crate::ai::error::AiErrorCode::Cancelled {
                "cancelled"
            } else {
                "failed"
            };
            repository::finish_turn(pool, turn_id, state, Some(&message)).await?;
            if !crate::external_agents::conversation::is_active(conversation_id) {
                let run_id: String = sqlx::query_scalar(
                    "SELECT run_id FROM spaces_external_agent_conversations WHERE id=$1",
                )
                .bind(conversation_id)
                .fetch_one(pool)
                .await
                .map_err(db::sql_error)?;
                mark_disconnected(pool, conversation_id, &run_id, &message).await?;
            }
            Err(message)
        }
    }
}
