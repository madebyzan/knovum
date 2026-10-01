use tauri::AppHandle;

use super::{completion, driver, identity::TaskIds};
use crate::{
    ai::contracts::ExecutionCapability,
    spaces::assistant::{repository, tools, StartAssistantTaskRequest},
};

pub(super) async fn recover_all(app: AppHandle) -> Result<(), String> {
    let pool = crate::spaces::db::pool(&app).await?;
    for record in repository::recoverable_tasks(pool).await? {
        recover_record(&app, pool, record).await?;
    }
    Ok(())
}

pub(super) async fn recover_one(app: AppHandle, run_id: &str) -> Result<(), String> {
    let pool = crate::spaces::db::pool(&app).await?;
    let record = repository::recoverable_task(pool, run_id).await?;
    recover_record(&app, pool, record).await
}

async fn recover_record(
    app: &AppHandle,
    pool: &'static sqlx::SqlitePool,
    record: repository::RecoverableTask,
) -> Result<(), String> {
    if record.profile_version != super::ASSISTANT_PROFILE_VERSION {
        completion::fail(
            pool,
            &record.run_id,
            "Assistant profile changed during recovery",
        )
        .await?;
        return Ok(());
    }
    let catalog = tools::project(record.mode)?;
    if catalog.digest != record.catalog_digest || catalog.digest != record.checkpoint.catalog_digest
    {
        completion::fail(
            pool,
            &record.run_id,
            "Assistant action catalog changed during recovery",
        )
        .await?;
        return Ok(());
    }
    let capability = if record.requires_vision {
        ExecutionCapability::VisionUnderstanding
    } else {
        ExecutionCapability::TextGeneration
    };
    let target = crate::ai::settings::resolve(app, record.model_binding.clone(), capability)
        .map_err(|error| error.to_string())?;
    let request = StartAssistantTaskRequest {
        space_id: record.space_id,
        conversation_id: Some(record.conversation_id.clone()),
        branch_id: Some(record.branch_id.clone()),
        prompt: record.prompt,
        mode: record.mode,
        target: record.model_binding,
        max_turns: record.checkpoint.max_model_turns,
        max_tool_calls: record.checkpoint.max_tool_calls,
        max_total_tokens: record.max_total_tokens,
        max_duration_seconds: record.max_duration_seconds,
        attachments: Vec::new(),
    };
    let ids = TaskIds {
        task_id: record.task_id,
        run_id: record.run_id,
        conversation_id: record.conversation_id,
        branch_id: record.branch_id,
        grant_id: record.grant_id,
    };
    let mut checkpoint = record.checkpoint;
    let run = crate::spaces::runs::load_run(pool, &ids.run_id).await?;
    if matches!(
        run.state,
        crate::spaces::runs::RunState::Waiting | crate::spaces::runs::RunState::Queued
    ) {
        crate::spaces::runs::transition_run(
            pool,
            &ids.run_id,
            run.state_version,
            crate::spaces::runs::RunState::Running,
            None,
        )
        .await?;
    }
    let cancellation = super::cancellation::register(&ids.run_id)?;
    let active_run_id = ids.run_id.clone();
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        match driver::drive(
            driver::DriverContext {
                app: &app,
                pool,
                target: &target,
                request: &request,
                ids: &ids,
                catalog: &catalog,
                updates: None,
            },
            &mut checkpoint,
            cancellation,
        )
        .await
        {
            Ok(output) => {
                if let Err(error) = completion::complete(pool, &ids, output).await {
                    log::error!("Recovered Assistant completion failed: {error}");
                    if let Err(write_error) = completion::fail(pool, &ids.run_id, &error).await {
                        log::error!("Recovered Assistant failure recording failed: {write_error}");
                    }
                }
            }
            Err(error) => {
                if let Err(write_error) = completion::fail(pool, &ids.run_id, &error.message).await
                {
                    log::error!("Recovered Assistant failure recording failed: {write_error}");
                }
                log::error!("Recovered Assistant task failed: {error}");
            }
        }
        super::cancellation::remove(&active_run_id);
    });
    Ok(())
}
