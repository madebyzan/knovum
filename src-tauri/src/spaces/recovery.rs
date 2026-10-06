use crate::app_error::AppError;
use tauri::{AppHandle, Manager};
use tokio::sync::OnceCell;

#[derive(Default)]
pub(super) struct SpaceRuntimeInitialization(OnceCell<Result<(), String>>);

pub(super) fn start(app: AppHandle) {
    app.manage(SpaceRuntimeInitialization::default());
    tauri::async_runtime::spawn(async move {
        match ready(&app).await {
            Ok(()) => {
                super::runs::start_scheduler(app.clone());
                super::knowledge::retrieval::retired_generations::start(app.clone());
                super::assets::enrichment::jobs::scheduler::start(app.clone());
                super::conversations::recover_tasks(app.clone());
                super::external_conversations::recover(app);
            }
            Err(error) => log::error!("Space runtime initialization failed: {error}"),
        }
    });
}

#[tauri::command]
pub async fn await_space_runtime(app: AppHandle) -> Result<(), AppError> {
    Ok(ready(&app).await?)
}

async fn ready(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<SpaceRuntimeInitialization>();
    state.0.get_or_init(|| recover(app)).await.clone()
}

async fn recover(app: &AppHandle) -> Result<(), String> {
    let pool = super::db::pool(app).await?;
    super::knowledge::retrieval::invalidation_outbox::drain(app, pool).await?;
    super::assets::reconcile_interrupted_work(app).await?;
    super::assets::import::mcp_resource::discard_interrupted(app)?;
    super::speech::reconcile_interrupted(app).await?;
    super::agent_artifacts::recovery::discard_orphaned_workspaces(app).await?;
    super::derivation::reconcile_interrupted(app).await?;
    super::package::reconcile(app).await?;
    super::conversations::tool_operations::recover(pool).await?;
    let retained: Vec<String> = sqlx::query_scalar("SELECT json_extract(input_snapshot,'$.extensions.id') FROM spaces_runs WHERE completed_at IS NULL AND json_extract(input_snapshot,'$.extensions.id') IS NOT NULL")
        .fetch_all(pool).await.map_err(super::db::sql_error)?;
    crate::extensions::recover(app, &retained).await?;
    Ok(())
}
