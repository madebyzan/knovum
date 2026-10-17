use std::path::PathBuf;

use serde_json::Value;
use tauri::AppHandle;

use super::GenerateVisualizationRequest;
use crate::ai::capabilities::image;
use crate::ai::contracts::{ExternalAgentKind, ImageReference};
use crate::ai::models::ModelExecutionTarget;
use crate::ai::tasks::TaskExecutionTarget;
use crate::foundation::app_error::AppError;
use crate::spaces::derivation::scratch_file::ScratchFile;
use crate::spaces::derivation::source_inputs::{self, RenderedImage};
use crate::spaces::knowledge::retrieval::hybrid::{self, HybridSearchRequest};
use crate::spaces::{assets::asset_directory, db};

/// What the selected resources turn into for one image generation: the pixels,
/// delivered the way this executor reads them, plus the text for the prompt.
pub(super) struct PreparedSources {
    pub delivery: Delivery,
    pub grounding: Vec<Value>,
}

/// The executor and its pixels travel together, so a model can never be handed
/// files and an Agent can never be handed a request body.
pub(super) enum Delivery {
    Model {
        target: ModelExecutionTarget,
        references: Vec<ImageReference>,
    },
    Agent {
        runtime_kind: ExternalAgentKind,
        attachments: Vec<PathBuf>,
        /// The Run owns the rendered files until the Agent is done with them.
        _files: Vec<ScratchFile>,
    },
}

impl PreparedSources {
    pub fn is_empty(&self) -> bool {
        self.attached_images() == 0 && self.grounding.is_empty()
    }

    pub fn attached_images(&self) -> usize {
        match &self.delivery {
            Delivery::Model { references, .. } => references.len(),
            Delivery::Agent { attachments, .. } => attachments.len(),
        }
    }
}

pub(super) async fn prepare(
    app: &AppHandle,
    request: &GenerateVisualizationRequest,
    target: &TaskExecutionTarget,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<PreparedSources, AppError> {
    let rendered = render(app, request, target).await?;
    let grounding = grounding(app, request, &rendered, cancellation).await?;
    let delivery = match target {
        TaskExecutionTarget::Model(target) => Delivery::Model {
            target: target.clone(),
            references: rendered
                .into_iter()
                .map(|image| ImageReference {
                    png: image.png,
                    width: image.width,
                    height: image.height,
                })
                .collect(),
        },
        TaskExecutionTarget::Agent { runtime_kind } => {
            let directory = asset_directory(app)?;
            let (attachments, files) =
                source_inputs::write_png_files(&rendered, &directory, "image-reference").await?;
            Delivery::Agent {
                runtime_kind: *runtime_kind,
                attachments,
                _files: files,
            }
        }
    };
    Ok(PreparedSources {
        delivery,
        grounding,
    })
}

/// Render the selected images within the budget the executor declared. A model
/// that only reads words renders nothing: those sources fall back to the text
/// the Space has indexed for them.
async fn render(
    app: &AppHandle,
    request: &GenerateVisualizationRequest,
    target: &TaskExecutionTarget,
) -> Result<Vec<RenderedImage>, AppError> {
    let (max_images, max_bytes) = match target {
        TaskExecutionTarget::Model(model) => match image::reference_support(model) {
            Some(support) => (usize::from(support.max_images), support.max_bytes_per_image),
            None => return Ok(Vec::new()),
        },
        // An Agent reads files from disk: the renderer's own ceiling is the limit.
        TaskExecutionTarget::Agent { .. } => (request.inputs.len(), crate::images::MAX_PNG_BYTES),
    };
    let rendered = source_inputs::render_images(
        db::pool(app).await?,
        &request.space_id,
        &request.inputs,
        asset_directory(app)?.as_path(),
        max_bytes,
    )
    .await?;
    if rendered.len() > max_images {
        return Err(AppError::invalid(format!(
            "The configured image model accepts at most {max_images} reference images"
        )));
    }
    Ok(rendered)
}

/// Retrieve text for the sources that are not travelling as pixels. With no
/// such source there is nothing to index, so the Run skips retrieval entirely.
async fn grounding(
    app: &AppHandle,
    request: &GenerateVisualizationRequest,
    rendered: &[RenderedImage],
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<Vec<Value>, AppError> {
    let textual: Vec<String> = request
        .inputs
        .iter()
        .filter(|input| !rendered.iter().any(|image| image.node_id == input.node_id))
        .map(|input| input.node_id.clone())
        .collect();
    if textual.is_empty() {
        return Ok(Vec::new());
    }
    crate::spaces::knowledge::retrieval::rebuild::prepare_space_index(
        app,
        &request.space_id,
        Some(textual.clone()),
        &request.id,
        cancellation,
    )
    .await?;
    let retrieval = hybrid::search(
        app,
        HybridSearchRequest {
            semantic: true,
            space_id: Some(request.space_id.clone()),
            query: request.intent.clone(),
            node_ids: Some(textual),
        },
        cancellation,
    )
    .await?;
    Ok(crate::spaces::knowledge::grounding::prompt::select_sources(
        retrieval.matches,
    )?)
}
