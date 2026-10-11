use std::path::PathBuf;

use tauri::{AppHandle, Manager, Runtime};

use crate::foundation::app_error::{AppError, AppResult};

pub(crate) fn root<R: Runtime>(app: &AppHandle<R>) -> AppResult<PathBuf> {
    app.path()
        .app_data_dir()
        .map(|root| root.join("agent-workspaces").join("spaces"))
        .map_err(|error| AppError::internal(error.to_string()))
}

pub(crate) fn space_directory<R: Runtime>(
    app: &AppHandle<R>,
    space_id: &str,
) -> AppResult<PathBuf> {
    let space = uuid::Uuid::parse_str(space_id)
        .map_err(|_| AppError::invalid("Agent session requires a valid Space identifier"))?;
    Ok(root(app)?.join(space.to_string()))
}

pub(crate) fn session_directory<R: Runtime>(
    app: &AppHandle<R>,
    space_id: &str,
    session_id: &str,
) -> AppResult<PathBuf> {
    let session = uuid::Uuid::parse_str(session_id)
        .map_err(|_| AppError::invalid("Agent session requires a valid session identifier"))?;
    Ok(space_directory(app, space_id)?.join(session.to_string()))
}
