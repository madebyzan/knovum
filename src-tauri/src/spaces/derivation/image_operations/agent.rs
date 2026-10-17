use tauri::{AppHandle, Manager};
use tokio_util::sync::CancellationToken;

use super::{contracts::ImageOperation, source::SourceImage};
use crate::foundation::app_error::AppError;
use crate::{
    ai::contracts::ExternalAgentKind,
    external_agents::task::{self, ArtifactFormat},
    foundation::scratch_file::ScratchFile,
};

pub(super) async fn execute(
    app: &AppHandle,
    kind: ExternalAgentKind,
    operation: &ImageOperation,
    source: &SourceImage,
    cancellation: &CancellationToken,
) -> Result<Vec<u8>, AppError> {
    let directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("image-operation-inputs");
    tokio::fs::create_dir_all(&directory)
        .await
        .map_err(|error| error.to_string())?;
    let image = ScratchFile::new(directory.join(format!("{}.png", uuid::Uuid::new_v4())));
    tokio::fs::write(image.path(), &source.bytes)
        .await
        .map_err(|error| error.to_string())?;
    let instruction = match operation {
        ImageOperation::MultiAngle { .. } => operation.view_prompt().expect("multi-angle has a view prompt"),
        ImageOperation::RemoveObjectPrompt { prompt } => format!("Remove the following object(s) from the input image and naturally complete their background. Preserve unselected content and dimensions: {prompt}"),
        _ => return Err("This image operation requires a local model".into()),
    };
    task::artifact(
        app,
        kind,
        format!(
            "{instruction}\nCreate the edited result as PNG using your own available tools/Skills."
        ),
        vec![image.path().to_owned()],
        ArtifactFormat::Png,
        cancellation.clone(),
    )
    .await
}

pub(crate) fn discard_interrupted(app: &AppHandle) -> Result<(), String> {
    let directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("image-operation-inputs");
    crate::external_agents::artifacts::discard(&directory).map_err(|error| error.to_string())
}
