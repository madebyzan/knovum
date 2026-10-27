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
    let extensions = crate::extensions::snapshots::prepare(
        &app,
        &request.extensions,
        &crate::spaces::mcp::local_tool_names(),
        None,
    )
    .await?;
    let snapshot_id = extensions.id.clone();
    let mut turn = match external_conversations::start_for_conversation(
        app.clone(),
        external_conversations::StartExternalConversationRequest {
            space_id: request.space_id,
            runtime_kind,
            title: prompt.chars().take(80).collect(),
            max_duration_seconds: 900,
            input_snapshot: serde_json::json!({"prompt": prompt, "extensions": extensions, "attachments": context.manifest, "spaceContext": context.space}),
        },
        conversation.clone(),
        updates.clone(),
    )
    .await
    {
        Ok(turn) => turn,
        Err(error) => {
            crate::extensions::reclamation::release(&app, &snapshot_id).await?;
            return Err(error);
        }
    };
    // An Agent that already holds the conversation is sent the new request alone; a new
    // runtime is given the stored history to continue from.
    let begun = async {
        let history = match (&conversation, turn.continues()) {
            (Some((id, branch)), false) => repository::build_context(pool, id, branch).await?,
            _ => None,
        };
        let prepared = context.agent_prompt(
            &app,
            &prompt,
            history.as_deref(),
            !extensions.skills.is_empty(),
        )?;
        let turn_id = external_conversations::begin_turn_for_conversation(
            app.clone(),
            &turn.binding.id,
            &prompt,
            &context.message_attachments,
        )
        .await?;
        Ok::<_, String>((turn_id, prepared))
    }
    .await;
    let (turn_id, prepared) = match begun {
        Ok(begun) => begun,
        Err(error) => {
            if let Err(settlement) =
                external_conversations::fail_for_conversation(&app, &turn, &error).await
            {
                log::error!("Agent task settlement failed: {settlement}");
            }
            drop(turn);
            crate::extensions::reclamation::release(&app, &snapshot_id).await?;
            return Err(error);
        }
    };
    let accepted = ConversationTaskAccepted {
        task_id: turn.binding.id.clone(),
        run_id: turn.binding.run_id.clone(),
        conversation_id: turn.binding.conversation_id.clone(),
        branch_id: turn.binding.branch_id.clone(),
    };
    let run_id = turn.binding.run_id.clone();
    tauri::async_runtime::spawn(async move {
        let outcome = external_conversations::send_for_conversation(
            app.clone(),
            &mut turn,
            &turn_id,
            prepared,
        )
        .await;
        let outcome = match outcome {
            Ok(()) => external_conversations::close_for_conversation(app.clone(), &turn).await,
            Err(error) => Err(error),
        };
        let settlement_error = if let Err(error) = outcome {
            external_conversations::fail_for_conversation(&app, &turn, &error)
                .await
                .err()
                .map(|settlement| {
                    log::error!(
                        "External Agent task settlement failed for run {run_id}: {settlement}; original failure: {error}"
                    );
                    format!(
                        "External Agent failed: {error}; its failure could not be recorded: {settlement}"
                    )
                })
        } else {
            None
        };
        // The conversation's runtime is free for its next turn before this one reports.
        drop(turn);
        let (state, terminal_error) = match db::pool(&app).await {
            Ok(pool) => match crate::spaces::runs::load_run(pool, &run_id).await {
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
                            run_id
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
                        run_id
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
                    run_id
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
