mod citations;
mod images;
mod recognition;
mod source;

use std::path::Path;

use serde_json::Value;
use tokio_util::sync::CancellationToken;

use super::files;
use crate::documents::generation::{worker, DocumentFormat};
use crate::foundation::app_error::AppError;

/// Serializes the resource into the requested format. The engine owns parsing
/// and writing; this owns authorization, the attached images and the check that
/// every file it read is still the one the user is looking at.
pub(super) async fn write(
    app: &tauri::AppHandle,
    node_id: &str,
    payload: &Value,
    format: DocumentFormat,
    destination: &Path,
    cancellation: &CancellationToken,
) -> Result<(), AppError> {
    let input = source::load(app, node_id, payload, format).await?;
    let snapshots = input.snapshots();
    let bytes = worker::render(format, input, cancellation).await?;
    crate::foundation::blocking::run({
        let app = app.clone();
        let destination = destination.to_owned();
        move || {
            for snapshot in snapshots {
                snapshot.verify_current()?;
            }
            files::write(&app, &destination, &bytes)
        }
    })
    .await?;
    Ok(())
}
