use crate::command_error::CommandError;
use tauri::AppHandle;

use crate::ai::capabilities;

#[tauri::command]
pub async fn stage_generated_artifact_import(
    app: AppHandle,
    space_id: String,
    session_id: String,
    relative_path: String,
) -> Result<crate::spaces::assets::import::PreparedSpaceImport, CommandError> {
    let completed = super::session::completed_at(&app, &space_id, &session_id).await?;
    if completed.is_none() {
        return Err(("Agent artifact session is not ready for import".to_owned()).into());
    }
    let workspace = capabilities::agent_artifacts::workspace(&app, &space_id, &session_id)
        .map_err(|error| error.to_string())?;
    let artifact = capabilities::agent_artifacts::resolve(&workspace, &relative_path)
        .map_err(|error| error.to_string())?;
    Ok((crate::blocking::run(move || {
        crate::spaces::assets::import::prepare_owned_asset(&app, artifact)
    })
    .await)?)
}
