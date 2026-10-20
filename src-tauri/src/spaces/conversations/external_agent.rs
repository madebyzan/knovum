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
    let extensions = crate::extensions::snapshots::prepare(
        &app,
        &request.extensions,
        &crate::spaces::mcp::local_tool_names(),
        None,
    )
    .await?;
    let snapshot_id = extensions.id.clone();
    let session = match external_conversations::start_for_conversation(
        app.clone(),
        external_conversations::StartExternalConversationRequest {
            space_id: request.space_id,
            runtime_kind,
            title: prompt.chars().take(80).collect(),
            max_duration_seconds: 900,
            input_snapshot: serde_json::json!({"prompt": prompt, "extensions": extensions, "attachments": context.manifest, "spaceContext": context.space}),
        },
        conversation,
        updates.clone(),
    )
    .await {
        Ok(session) => session,
        Err(error) => { crate::extensions::reclamation::release(&app, &snapshot_id).await?; return Err(error); }
    };
    let turn_id = match external_conversations::begin_turn_for_conversation(
        app.clone(),
        &session.id,
        &prompt,
        &context.message_attachments,
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
            crate::extensions::reclamation::release(&app, &snapshot_id).await?;
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
        let settlement_error = if let Err(error) = outcome {
            external_conversations::fail_for_conversation(
                &app,
                &session.id,
                &session.run_id,
                &error,
            )
            .await
            .err()
            .map(|settlement| {
                log::error!(
                    "External Agent task settlement failed for run {}: {settlement}; original failure: {error}",
                    session.run_id
                );
                format!(
                    "External Agent failed: {error}; its failure could not be recorded: {settlement}"
                )
            })
        } else {
            None
        };
        let (state, terminal_error) = match db::pool(&app).await {
            Ok(pool) => match crate::spaces::runs::load_run(pool, &session.run_id).await {
                Ok(run) => match run.state {
                    RunState::Succeeded => (
                        super::contracts::ConversationTaskFinishedState::Succeeded,
                        None,
                    ),
                    RunState::CompletedWithErrors => (
                        super::contracts::ConversationTaskFinishedState::CompletedWithErrors,
                        None,
                    ),
                    RunState::Failed => (
                        super::contracts::ConversationTaskFinishedState::Failed,
                        None,
                    ),
                    RunState::Cancelled => (
                        super::contracts::ConversationTaskFinishedState::Cancelled,
                        None,
                    ),
                    state @ (RunState::Queued
                    | RunState::Running
                    | RunState::Waiting
                    | RunState::Paused) => {
                        let state_error = format!(
                            "External Agent producer finished while run {} remained in non-terminal state {state:?}",
                            session.run_id
                        );
                        log::error!("{state_error}");
                        (
                            super::contracts::ConversationTaskFinishedState::SettlementFailed,
                            Some(match settlement_error {
                                Some(settlement) => format!("{settlement}; {state_error}"),
                                None => state_error,
                            }),
                        )
                    }
                },
                Err(error) => {
                    log::error!(
                        "External Agent final state unavailable for run {}: {error}",
                        session.run_id
                    );
                    (
                        super::contracts::ConversationTaskFinishedState::SettlementFailed,
                        Some(match settlement_error {
                            Some(settlement) => {
                                format!("{settlement}; final state could not be read: {error}")
                            }
                            None => {
                                format!("External Agent final state could not be read: {error}")
                            }
                        }),
                    )
                }
            },
            Err(error) => {
                log::error!(
                    "External Agent database unavailable while settling run {}: {error}",
                    session.run_id
                );
                (
                    super::contracts::ConversationTaskFinishedState::SettlementFailed,
                    Some(match settlement_error {
                        Some(settlement) => format!(
                            "{settlement}; database was unavailable during settlement: {error}"
                        ),
                        None => format!(
                            "External Agent database was unavailable during settlement: {error}"
                        ),
                    }),
                )
            }
        };
        if let Err(error) = crate::extensions::reclamation::release(&app, &snapshot_id).await {
            log::error!("Agent extension release failed: {error}");
        }
        let _ = updates.send(ConversationTaskUpdate::Finished {
            state,
            error: terminal_error,
        });
    });
    Ok(accepted)
}
