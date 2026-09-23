use std::{path::PathBuf, sync::Arc};

use tauri::AppHandle;

use super::{repository, StartExternalConversationRequest};
use crate::{ai::capabilities, spaces::db};

mod lifecycle;

pub(super) use lifecycle::mark_disconnected;

pub async fn start(
    app: AppHandle,
    request: StartExternalConversationRequest,
) -> Result<super::ExternalConversationSummary, String> {
    lifecycle::validate(&request)?;
    crate::ai::settings::ensure_agent_enabled(&app, request.runtime_kind)
        .map_err(|error| error.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    let run_id = uuid::Uuid::new_v4().to_string();
    let workspace = capabilities::agent_artifacts::workspace(&app, &request.space_id, &id)
        .map_err(|error| error.to_string())?;
    capabilities::agent_artifacts::prepare(&workspace).map_err(|error| error.to_string())?;
    lifecycle::create_records(&app, &request, &id, &run_id).await?;
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
    let callback = super::updates::callback(app.clone(), id.clone(), run_id.clone());
    let keepalive: Arc<dyn Send + Sync> = binding.clone();
    let runtime_session = crate::external_agents::conversation::start(
        id.clone(),
        request.runtime_kind,
        workspace,
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
            mark_disconnected(db::pool(&app).await?, &id, &run_id, &error).await?;
            Err(error)
        }
    }
}

pub async fn send(
    app: AppHandle,
    conversation_id: &str,
    prompt: &str,
    attachments: Vec<PathBuf>,
) -> Result<(), String> {
    let prompt = prompt.trim();
    if prompt.is_empty() || prompt.chars().count() > 100_000 {
        return Err("external-conversation-prompt-invalid".into());
    }
    let prepared = crate::external_agents::prompt::AgentPrompt::from_renderer(
        &app,
        prompt.to_owned(),
        attachments,
    )
    .map_err(|error| error.to_string())?;
    let pool = db::pool(&app).await?;
    let turn_id = repository::start_turn(pool, conversation_id, prompt).await?;
    match crate::external_agents::conversation::send(conversation_id, prepared).await {
        Ok(()) => repository::finish_turn(pool, &turn_id, "completed", None).await,
        Err(error) => {
            let message = error.to_string();
            let state = if error.code == crate::ai::error::AiErrorCode::Cancelled {
                "cancelled"
            } else {
                "failed"
            };
            repository::finish_turn(pool, &turn_id, state, Some(&message)).await?;
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

pub async fn configure(
    app: AppHandle,
    conversation_id: &str,
    option_id: String,
    value: serde_json::Value,
) -> Result<super::ExternalConversationSummary, String> {
    if option_id.trim().is_empty() || option_id.chars().count() > 256 {
        return Err("external-session-option-invalid".into());
    }
    let options =
        crate::external_agents::conversation::configure(conversation_id, option_id, value)
            .await
            .map_err(|error| error.to_string())?;
    let pool = db::pool(&app).await?;
    repository::update_session_options(pool, conversation_id, options).await?;
    lifecycle::load(pool, conversation_id).await
}

pub async fn set_mode(
    app: AppHandle,
    conversation_id: &str,
    mode_id: String,
) -> Result<super::ExternalConversationSummary, String> {
    if mode_id.trim().is_empty() || mode_id.chars().count() > 256 {
        return Err("external-session-mode-invalid".into());
    }
    crate::external_agents::conversation::set_mode(conversation_id, mode_id.clone())
        .await
        .map_err(|error| error.to_string())?;
    let pool = db::pool(&app).await?;
    repository::update_session_mode(pool, conversation_id, &mode_id).await?;
    lifecycle::load(pool, conversation_id).await
}
