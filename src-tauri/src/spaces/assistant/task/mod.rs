mod cancellation;
mod completion;
pub(crate) mod context;
mod driver;
mod identity;
mod persistence;
mod recovery;
mod tool_execution;

use tauri::ipc::Channel;
use tauri::AppHandle;

use super::{repository, tools, AssistantTaskAccepted, StartAssistantTaskRequest};
use crate::spaces::{
    db,
    runs::{transition_run, RunState},
};

pub(super) const ASSISTANT_PROFILE_VERSION: u16 = 4;

pub(super) struct NewAssistantTask<'a> {
    pub task_id: &'a str,
    pub run_id: &'a str,
    pub conversation_id: &'a str,
    pub branch_id: &'a str,
    pub grant_id: &'a str,
    pub model_binding: &'a str,
    pub catalog_digest: &'a str,
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
    let target = crate::ai::settings::resolve(&app, request.target.clone(), capability)
        .map_err(|error| error.to_string())?;
    let catalog = tools::project(request.mode)?;
    let ids = identity::TaskIds::new(&request);
    let conversation_context =
        repository::build_context(pool, &ids.conversation_id, &ids.branch_id).await?;
    let now = crate::clock::unix_time_millis()?;
    let grant = identity::grant(&request, &ids, &catalog, now);
    let mut checkpoint = crate::agent_runtime::start(
        context
            .initial_message(&request.prompt, conversation_context.as_deref(), &target)
            .await?,
        request.max_turns,
        request.max_tool_calls,
        ids.run_id.clone(),
        uuid::Uuid::new_v4().to_string(),
        catalog.digest.clone(),
    )?;
    persistence::persist_task(
        pool,
        persistence::NewAssistantTaskState {
            request: &request,
            ids: &ids,
            grant: &grant,
            catalog: &catalog,
            context_manifest: &context.manifest,
            space_context: &context.space,
            attachment_summaries: &context.summaries,
            checkpoint: &checkpoint,
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
    let active_run_id = ids.run_id.clone();
    tauri::async_runtime::spawn(async move {
        match driver::drive(
            driver::DriverContext {
                app: &app,
                pool,
                target: &target,
                request: &request,
                ids: &ids,
                catalog: &catalog,
                updates: Some(&updates),
            },
            &mut checkpoint,
            cancellation,
        )
        .await
        {
            Ok(output) => {
                if let Err(error) = completion::complete(pool, &ids, output).await {
                    log::error!("Assistant task completion failed: {error}");
                    if let Err(write_error) = completion::fail(pool, &ids.run_id, &error).await {
                        log::error!("Assistant failure recording failed: {write_error}");
                    }
                    let _ = updates.send(super::AssistantTaskUpdate::Finished {
                        state: completion::stopped_state(pool, &ids.run_id).await,
                    });
                } else {
                    let _ = updates.send(super::AssistantTaskUpdate::Finished {
                        state: "succeeded".into(),
                    });
                }
            }
            Err(error) => {
                if let Err(write_error) = completion::fail(pool, &ids.run_id, &error).await {
                    log::error!("Assistant failure recording failed: {write_error}");
                }
                let _ = updates.send(super::AssistantTaskUpdate::Finished {
                    state: completion::stopped_state(pool, &ids.run_id).await,
                });
                log::error!("Assistant task failed: {error}");
            }
        }
        cancellation::remove(&active_run_id);
    });
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
    recovery::recover_one(app, run_id).await
}

pub(super) async fn require_paused_checkpoint(
    pool: &sqlx::SqlitePool,
    run_id: &str,
) -> Result<(), String> {
    let ready: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM spaces_runs r WHERE r.id=$1 AND r.state='paused' AND r.pause_requested_at IS NOT NULL
         AND EXISTS (SELECT 1 FROM spaces_run_events e WHERE e.run_id=r.id
           AND e.kind='assistant_paused_checkpoint' AND e.created_at>=r.pause_requested_at)",
    ).bind(run_id).fetch_optional(pool).await.map_err(db::sql_error)?;
    ready.map(|_| ()).ok_or_else(||
        "assistant-pause-settling: wait for the current model or tool operation to reach a durable checkpoint".into()
    )
}
