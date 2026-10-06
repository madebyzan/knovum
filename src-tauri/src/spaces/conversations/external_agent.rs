use tauri::{ipc::Channel, AppHandle};

use super::{
    context, repository, ConversationTaskAccepted, ConversationTaskUpdate,
    SubmitConversationTaskRequest,
};
use crate::{
    ai::contracts::ExternalAgentKind,
    spaces::{db, external_conversations, runs::RunState},
};

pub async fn start(
    app: AppHandle,
    request: SubmitConversationTaskRequest,
    runtime_kind: ExternalAgentKind,
    updates: Channel<ConversationTaskUpdate>,
) -> Result<ConversationTaskAccepted, String> {
    if !matches!(request.mode, super::ConversationMode::Execute) {
        return Err(
            "external-agent-mode-unsupported: desktop Agent sessions use Execute mode".into(),
        );
    }
    let prompt = request.prompt.trim().to_owned();
    if request.space_id.trim().is_empty() || prompt.is_empty() || prompt.chars().count() > 100_000 {
        return Err("conversation-request-invalid: Space and prompt are required".into());
    }
    let conversation = match (request.conversation_id, request.branch_id) {
        (None, None) => None,
        (Some(id), Some(branch)) if !id.is_empty() && !branch.is_empty() => Some((id, branch)),
        _ => {
            return Err(
                "conversation-invalid: conversation and branch must be selected together".into(),
            )
        }
    };
    let pool = db::pool(&app).await?;
    if let Some((id, _)) = &conversation {
        let owned: Option<i64> =
            sqlx::query_scalar("SELECT 1 FROM spaces_conversations WHERE id=$1 AND space_id=$2")
                .bind(id)
                .bind(&request.space_id)
                .fetch_optional(pool)
                .await
                .map_err(db::sql_error)?;
        if owned.is_none() {
            return Err("conversation-unavailable: conversation belongs to another Space".into());
        }
    }
    let context = context::resolve(
        &app,
        pool,
        &request.space_id,
        &request.attachments,
        context::ContextTarget::ExternalAgent,
    )
    .await?;
    let history = if let Some((id, branch)) = &conversation {
        repository::build_context(pool, id, branch).await?
    } else {
        None
    };
    let prepared = context.agent_prompt(&app, &prompt, history.as_deref())?;
    let session = external_conversations::start_for_conversation(
        app.clone(),
        external_conversations::StartExternalConversationRequest {
            space_id: request.space_id,
            runtime_kind,
            title: prompt.chars().take(80).collect(),
            max_duration_seconds: 900,
            input_snapshot: serde_json::json!({"prompt": prompt, "attachments": context.manifest, "spaceContext": context.space}),
        },
        conversation,
        updates.clone(),
    )
    .await?;
    let turn_id = match external_conversations::begin_turn_for_conversation(
        app.clone(),
        &session.id,
        &prompt,
        &context.summaries,
    )
    .await
    {
        Ok(id) => id,
        Err(error) => {
            if let Err(settlement) = external_conversations::fail_for_conversation(
                &app,
                &session.id,
                &session.run_id,
                &error,
            )
            .await
            {
                log::error!("Agent task settlement failed: {settlement}");
            }
            return Err(error);
        }
    };
    let accepted = ConversationTaskAccepted {
        task_id: session.id.clone(),
        run_id: session.run_id.clone(),
        conversation_id: session.conversation_id.clone(),
        branch_id: session.branch_id.clone(),
    };
    tauri::async_runtime::spawn(async move {
        let outcome = external_conversations::send_for_conversation(
            app.clone(),
            &session.id,
            &turn_id,
            prepared,
        )
        .await;
        let outcome = match outcome {
            Ok(()) => {
                external_conversations::close_for_conversation(app.clone(), &session.id).await
            }
            Err(error) => Err(error),
        };
        if let Err(error) = outcome {
            if let Err(settlement) = external_conversations::fail_for_conversation(
                &app,
                &session.id,
                &session.run_id,
                &error,
            )
            .await
            {
                log::error!("Agent task settlement failed: {settlement}");
            }
        }
        let state = match db::pool(&app).await {
            Ok(pool) => match crate::spaces::runs::load_run(pool, &session.run_id).await {
                Ok(run) if run.state == RunState::Succeeded => {
                    super::contract::ConversationTaskFinishedState::Succeeded
                }
                Ok(run) if run.state == RunState::Cancelled => {
                    super::contract::ConversationTaskFinishedState::Cancelled
                }
                _ => super::contract::ConversationTaskFinishedState::Failed,
            },
            Err(_) => super::contract::ConversationTaskFinishedState::Failed,
        };
        let _ = updates.send(ConversationTaskUpdate::Finished { state });
    });
    Ok(accepted)
}
