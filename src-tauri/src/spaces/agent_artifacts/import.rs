use crate::app_error::AppError;
use tauri::AppHandle;

#[tauri::command]
pub async fn stage_generated_artifact_import(
    app: AppHandle,
    space_id: String,
    session_id: String,
    relative_path: String,
) -> Result<crate::spaces::assets::import::prepare::PreparedSpaceImport, AppError> {
    let completed = super::session::completed_at(&app, &space_id, &session_id).await?;
    if completed.is_none() {
        return Err(("Agent artifact session is not ready for import".to_owned()).into());
    }
    let workspace =
        crate::spaces::agent_artifacts::workspace::session_directory(&app, &space_id, &session_id)
            .map_err(|error| error.to_string())?;
    let artifact = crate::external_agents::artifacts::resolve(&workspace, &relative_path)
        .map_err(|error| error.to_string())?;
    Ok((crate::blocking::run(move || {
        crate::spaces::assets::import::prepare::prepare_owned_asset(
            &app,
            artifact,
            tokio_util::sync::CancellationToken::new(),
        )
    })
    .await)?)
}
