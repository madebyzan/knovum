use std::path::PathBuf;

use serde_json::Value;
use tauri::AppHandle;

use super::GenerateVisualizationRequest;
use crate::ai::capabilities::image;
use crate::ai::contracts::ImageReference;
use crate::ai::tasks::TaskExecutionTarget;
use crate::foundation::app_error::AppError;
use crate::spaces::derivation::scratch_file::ScratchFile;
use crate::spaces::derivation::source_inputs::{self, RenderedImage};
use crate::spaces::knowledge::retrieval::hybrid::{self, HybridSearchRequest};
use crate::spaces::{assets::asset_directory, db};

/// An external Agent reads references from disk, so its budget is the file
/// budget rather than a provider request budget.
const AGENT_MAX_REFERENCES: usize = 10;
const AGENT_MAX_REFERENCE_BYTES: usize = 16 * 1024 * 1024;

/// What the selected resources turn into for one image generation: pixels for
/// the model, text for the prompt, and files for an Agent that reads from disk.
pub(super) struct PreparedSources {
    pub references: Vec<ImageReference>,
    pub attachments: Vec<PathBuf>,
    pub grounding: Vec<Value>,
    /// Owns the Agent's reference files until the Run is finished with them.
    _files: Vec<ScratchFile>,
}

impl PreparedSources {
    pub fn is_empty(&self) -> bool {
        self.references.is_empty() && self.attachments.is_empty() && self.grounding.is_empty()
    }
}

pub(super) async fn prepare(
    app: &AppHandle,
    request: &GenerateVisualizationRequest,
    target: &TaskExecutionTarget,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<PreparedSources, AppError> {
    let budget = match target {
        TaskExecutionTarget::Model(model) => image::reference_budget(model),
        TaskExecutionTarget::Agent { .. } => {
            Some((AGENT_MAX_REFERENCES, AGENT_MAX_REFERENCE_BYTES))
        }
    };
    let rendered = match budget {
        Some((max_images, max_bytes)) => {
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
            rendered
        }
        // A text-only image model cannot see pixels: those sources fall back to
        // whatever text the Space has indexed for them.
        None => Vec::new(),
    };
    let grounding = grounding(app, request, &rendered, cancellation).await?;
    match target {
        TaskExecutionTarget::Model(_) => Ok(PreparedSources {
            references: references(rendered),
            attachments: Vec::new(),
            grounding,
            _files: Vec::new(),
        }),
        TaskExecutionTarget::Agent { .. } => {
            let (attachments, files) = write_attachments(app, rendered).await?;
            Ok(PreparedSources {
                references: Vec::new(),
                attachments,
                grounding,
                _files: files,
            })
        }
    }
}

/// One selected image or several: the prompt says what to do with them.
fn references(rendered: Vec<RenderedImage>) -> Vec<ImageReference> {
    rendered
        .into_iter()
        .map(|image| ImageReference {
            png: image.png,
            width: image.width,
            height: image.height,
        })
        .collect()
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

async fn write_attachments(
    app: &AppHandle,
    rendered: Vec<RenderedImage>,
) -> Result<(Vec<PathBuf>, Vec<ScratchFile>), AppError> {
    let directory = asset_directory(app)?;
    let mut attachments = Vec::with_capacity(rendered.len());
    let mut files = Vec::with_capacity(rendered.len());
    for image in rendered {
        let path = directory.join(format!("image-reference-{}.png", uuid::Uuid::new_v4()));
        let file = ScratchFile::new(path.clone());
        tokio::fs::write(&path, &image.png)
            .await
            .map_err(|error| error.to_string())?;
        files.push(file);
        attachments.push(path);
    }
    Ok((attachments, files))
}
