mod cancellation;
mod completion;
pub(crate) mod context;
mod driver;
pub(super) mod identity;
mod incompatible_segments;
mod lifecycle;
mod model_recovery;
mod persistence;
mod recovery;
pub(super) mod tool_execution;

use tauri::ipc::Channel;
use tauri::AppHandle;

use super::{repository, tools, AssistantTaskAccepted, StartAssistantTaskRequest};
use crate::spaces::{
    db,
    runs::{transition_run, RunState},
};

pub(super) const ASSISTANT_PROFILE_VERSION: u16 = 5;

pub(super) struct NewAssistantTask<'a> {
    pub task_id: &'a str,
    pub run_id: &'a str,
    pub conversation_id: &'a str,
    pub branch_id: &'a str,
    pub grant_id: &'a str,
    pub model_binding: &'a str,
    pub catalog_digest: &'a str,
    pub session: &'a crate::spaces::assistant::execution::session::Session,
}

pub async fn start(
    app: AppHandle,
    request: StartAssistantTaskRequest,
    updates: Channel<super::AssistantTaskUpdate>,
) -> Result<AssistantTaskAccepted, String> {
    identity::validate(&request)?;
    let pool = db::pool(&app).await?;
    let context = context::resolve(
        &app,
        pool,
        &request.space_id,
        &request.attachments,
        context::ContextTarget::Model,
    )
    .await?;
    let capability = if context.requires_vision() {
        crate::ai::contracts::ExecutionCapability::VisionUnderstanding
    } else {
        crate::ai::contracts::ExecutionCapability::TextGeneration
    };
    let (target, profile) =
        crate::ai::settings::resolve_assistant(&app, request.target.clone(), capability)
            .map_err(|error| error.to_string())?;
    let catalog = tools::project(request.mode)?;
    let ids = identity::TaskIds::new(&request);
    let history = repository::load_history(pool, &ids.conversation_id, &ids.branch_id).await?;
    let now = crate::clock::unix_time_millis()?;
    let grant = identity::grant(&request, &ids, &catalog, now);
    let messages = vec![context.initial_message(ids.input_id(), &request.prompt, None)];
    let (session, messages) = persistence::persist_task(
        pool,
        persistence::NewAssistantTaskState {
            request: &request,
            ids: &ids,
            grant: &grant,
            catalog: &catalog,
            context_manifest: &context.manifest,
            space_context: &context.space,
            attachment_summaries: &context.summaries,
            target: &target,
            profile: &profile,
            messages: &messages,
            history: &history,
            now,
        },
    )
    .await?;
    transition_run(pool, &ids.run_id, 0, RunState::Running, None).await?;
    let accepted = AssistantTaskAccepted {
        task_id: ids.task_id.clone(),
        run_id: ids.run_id.clone(),
        conversation_id: ids.conversation_id.clone(),
        branch_id: ids.branch_id.clone(),
    };
    let cancellation = cancellation::register(&ids.run_id)?;
    lifecycle::launch(
        crate::spaces::assistant::execution::gateway::Gateway {
            app,
            pool: pool.clone(),
            target,
            profile,
            request,
            ids,
            catalog,
            session,
            updates: Some(updates),
            last_message: std::sync::Mutex::new(None),
        },
        messages,
        None,
        cancellation,
    );
    Ok(accepted)
}

pub(super) fn cancel(run_id: &str) -> Result<(), String> {
    cancellation::cancel(run_id)
}

pub(super) fn recover(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = recovery::recover_all(app).await {
            log::error!("Assistant recovery failed: {error}");
        }
    });
}

pub(super) async fn resume(app: AppHandle, run_id: &str) -> Result<(), String> {
    recovery::recover_one(app, run_id, true, None).await
}

pub(super) async fn retry_model(
    app: AppHandle,
    run_id: &str,
    expected_version: i64,
) -> Result<(), String> {
    recovery::recover_one(app, run_id, false, Some(expected_version)).await
}

#[cfg(test)]
mod tests;
