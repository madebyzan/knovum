use super::{contracts::ImageOperation, mask, source::SourceImage};
use crate::app_error::AppError;
use crate::{
    ai::contracts::AgentKind,
    external_agents::task::{self, ArtifactFormat},
    spaces::derivation::scratch_file::ScratchFile,
};
use tauri::{AppHandle, Manager};
use tokio_util::sync::CancellationToken;

pub(super) async fn execute(
    app: &AppHandle,
    kind: AgentKind,
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
    let mut sources = vec![image.path().to_owned()];
    let mask_file = if let ImageOperation::RemoveObject { strokes } = operation {
        let bytes = mask::encode_png(source.width, source.height, strokes, cancellation)?;
        let file = ScratchFile::new(directory.join(format!("{}.png", uuid::Uuid::new_v4())));
        tokio::fs::write(file.path(), bytes)
            .await
            .map_err(|error| error.to_string())?;
        sources.push(file.path().to_owned());
        Some(file)
    } else {
        None
    };
    let instruction = match operation {
        ImageOperation::Upscale { target_long_edge } => format!("Upscale the input image to a longest edge of at least {target_long_edge} pixels, preserving its aspect ratio and identity."),
        ImageOperation::MultiAngle { .. } => operation.view_prompt().expect("multi-angle has a view prompt"),
        ImageOperation::RemoveObject { .. } => "Remove the objects selected by white pixels in the second (mask) image. Inpaint the background naturally; preserve all unselected content and output dimensions. Do not erase to transparency.".into(),
        ImageOperation::RemoveObjectPrompt { prompt } => format!("Remove the following object(s) from the input image and naturally complete their background. Preserve unselected content and dimensions: {prompt}"),
    };
    let result = task::artifact(
        app,
        kind,
        format!(
            "{instruction}\nCreate the edited result as PNG using your own available tools/Skills."
        ),
        sources,
        ArtifactFormat::Png,
        cancellation.clone(),
    )
    .await;
    drop(mask_file);
    result
}

pub(crate) fn discard_interrupted(app: &AppHandle) -> Result<(), String> {
    let directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("image-operation-inputs");
    crate::ai::capabilities::agent_artifacts::discard(&directory).map_err(|error| error.to_string())
}
