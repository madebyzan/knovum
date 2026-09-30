use tauri::{ipc::Channel, AppHandle};

use super::{
    repository, task::context, AssistantTaskAccepted, AssistantTaskUpdate,
    SubmitAssistantTaskRequest,
};
use crate::{
    ai::contracts::AgentKind,
    spaces::{db, external_conversations, runs::RunState},
};

pub async fn start(
    app: AppHandle,
    request: SubmitAssistantTaskRequest,
    runtime_kind: AgentKind,
    updates: Channel<AssistantTaskUpdate>,
) -> Result<AssistantTaskAccepted, String> {
    let prompt = request.prompt.trim().to_owned();
    if request.space_id.trim().is_empty() || prompt.is_empty() || prompt.chars().count() > 100_000 {
        return Err("assistant-request-invalid: Space and prompt are required".into());
    }
    let conversation =
        match (request.conversation_id, request.branch_id) {
            (None, None) => None,
            (Some(id), Some(branch)) if !id.is_empty() && !branch.is_empty() => Some((id, branch)),
            _ => return Err(
                "assistant-conversation-invalid: conversation and branch must be selected together"
                    .into(),
            ),
        };
    let pool = db::pool(&app).await?;
    if let Some((id, _)) = &conversation {
        let owned: Option<i64> = sqlx::query_scalar(
            "SELECT 1 FROM spaces_assistant_conversations WHERE id=$1 AND space_id=$2",
        )
        .bind(id)
        .bind(&request.space_id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)?;
        if owned.is_none() {
            return Err(
                "assistant-conversation-unavailable: conversation belongs to another Space".into(),
            );
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
    let session = external_conversations::start_for_assistant(
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
    let turn_id = match external_conversations::begin_turn_for_assistant(
        app.clone(),
        &session.id,
        &prompt,
        &context.summaries,
    )
    .await
    {
        Ok(id) => id,
        Err(error) => {
            if let Err(settlement) = external_conversations::fail_for_assistant(
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
    let accepted = AssistantTaskAccepted {
        task_id: session.id.clone(),
        run_id: session.run_id.clone(),
        conversation_id: session.assistant_conversation_id.clone(),
        branch_id: session.branch_id.clone(),
    };
    tauri::async_runtime::spawn(async move {
        let outcome = external_conversations::send_for_assistant(
            app.clone(),
            &session.id,
            &turn_id,
            prepared,
        )
        .await;
        let outcome = match outcome {
            Ok(()) => external_conversations::close_for_assistant(app.clone(), &session.id).await,
            Err(error) => Err(error),
        };
        if let Err(error) = outcome {
            if let Err(settlement) = external_conversations::fail_for_assistant(
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
                Ok(run) if run.state == RunState::Succeeded => "succeeded",
                Ok(run) if run.state == RunState::Cancelled => "cancelled",
                _ => "failed",
            },
            Err(_) => "failed",
        };
        let _ = updates.send(AssistantTaskUpdate::Finished {
            state: state.into(),
        });
    });
    Ok(accepted)
}
