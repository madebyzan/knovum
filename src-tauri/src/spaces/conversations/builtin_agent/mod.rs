mod cancellation;
mod completion;
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

use super::{context, repository, tools, ConversationTaskAccepted, StartBuiltinAgentTaskRequest};
use crate::foundation::app_error::AppError;
use crate::spaces::{
    db,
    runs::{transition_run, RunState},
};

pub(super) const BUILTIN_AGENT_PROFILE_VERSION: u16 = 1;

pub(super) struct NewBuiltinAgentTask<'a> {
    pub task_id: &'a str,
    pub run_id: &'a str,
    pub conversation_id: &'a str,
    pub branch_id: &'a str,
    pub grant_id: &'a str,
    pub model_binding: &'a crate::ai::models::ModelTargetSelection,
    pub catalog_digest: &'a str,
    pub session: &'a crate::spaces::conversations::execution::session::Session,
}

pub async fn start(
    app: AppHandle,
    request: StartBuiltinAgentTaskRequest,
    updates: Channel<super::ConversationTaskUpdate>,
) -> Result<ConversationTaskAccepted, AppError> {
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
    let target =
        crate::ai::settings::resolve_builtin_agent(&app, request.target.clone(), capability)?;
    let profile =
        crate::ai::models::profile::resolve(&app, target.model_provider_kind()?, &target.model)
            .await;
    let mut catalog = tools::project()?;
    let local_tools = catalog
        .tools
        .iter()
        .map(|tool| tool.name.clone())
        .collect::<Vec<_>>();
    let extensions =
        crate::extensions::snapshots::prepare(&app, &request.extensions, &local_tools, None)
            .await?;
    let snapshot_id = extensions.id.clone();
    let started = async {
        catalog.extend(&extensions)?;
        let ids = identity::TaskIds::new(&request);
        let history = repository::load_history(pool, &ids.conversation_id, &ids.branch_id).await?;
        let now = crate::foundation::clock::unix_time_millis()?;
        let grant = identity::grant(&request, &ids, &catalog, now);
        let messages = vec![context.initial_message(ids.input_id(), &request.prompt, None)];
        let (session, messages) = persistence::persist_task(
            pool,
            persistence::NewBuiltinAgentTaskState {
                request: &request,
                ids: &ids,
                grant: &grant,
                catalog: &catalog,
                extensions: &extensions,
                context_manifest: &context.manifest,
                space_context: &context.space,
                message_attachments: &context.message_attachments,
                target: &target,
                profile: &profile,
                messages: &messages,
                history: &history,
                now,
            },
        )
        .await?;
        let launch_state = async {
            transition_run(pool, &ids.run_id, 0, RunState::Running, None).await?;
            cancellation::register(&ids.run_id)
        }
        .await;
        let cancellation = match launch_state {
            Ok(cancellation) => cancellation,
            Err(error) => {
                completion::fail(pool, &ids.run_id, &error).await?;
                return Err(error);
            }
        };
        let accepted = ConversationTaskAccepted {
            task_id: ids.task_id.clone(),
            run_id: ids.run_id.clone(),
            conversation_id: ids.conversation_id.clone(),
            branch_id: ids.branch_id.clone(),
        };
        lifecycle::launch(
            crate::spaces::conversations::execution::gateway::Gateway {
                app: app.clone(),
                pool: pool.clone(),
                target,
                profile,
                request,
                ids,
                catalog,
                extensions,
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
    .await;
    if started.is_err() {
        crate::extensions::reclamation::release(&app, &snapshot_id).await?;
    }
    started.map_err(AppError::from)
}

pub(super) fn cancel(run_id: &str) -> Result<(), String> {
    cancellation::cancel(run_id)
}

pub(super) fn recover(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = recovery::recover_all(app).await {
            log::error!("Builtin Agent recovery failed: {error}");
        }
    });
}

pub(super) async fn resume(app: AppHandle, run_id: &str) -> Result<(), String> {
    cancellation::wait_idle(run_id).await?;
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
