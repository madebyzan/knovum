use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::super::image_operations::contracts::ImageExecutionTarget;
use crate::local_resources::model_catalog::{LocalModelCapability, LocalModelId};

/// The frozen model each layer pass runs on. The canvas chooses these before the Run starts, so a
/// split never mixes two choices of one capability and a retry replays the same models.
#[derive(Clone, Deserialize, Serialize, JsonSchema, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct ImageLayersModels {
    pub saliency: ImageExecutionTarget,
    pub selection: ImageExecutionTarget,
    pub depth: ImageExecutionTarget,
    pub inpainting: ImageExecutionTarget,
}

impl ImageLayersModels {
    pub(crate) fn validate(&self) -> Result<(), String> {
        validate_local(&self.saliency, LocalModelCapability::ImageSegmentation)?;
        validate_local(&self.selection, LocalModelCapability::ImageSelection)?;
        validate_local(&self.depth, LocalModelCapability::ImageDepth)?;
        validate_local(&self.inpainting, LocalModelCapability::ImageInpainting)?;
        Ok(())
    }

    pub(crate) fn model_ids(&self) -> Result<[LocalModelId; 4], String> {
        Ok([
            local_model(&self.saliency)?,
            local_model(&self.selection)?,
            local_model(&self.depth)?,
            local_model(&self.inpainting)?,
        ])
    }
}

fn validate_local(
    target: &ImageExecutionTarget,
    capability: LocalModelCapability,
) -> Result<(), String> {
    let model = local_model(target)?;
    if model.capability() != capability {
        return Err("Image layer model does not serve its pass".into());
    }
    Ok(())
}

fn local_model(target: &ImageExecutionTarget) -> Result<LocalModelId, String> {
    match target {
        ImageExecutionTarget::Local { model, spec_id } => {
            if *spec_id != model.image_spec()? {
                return Err("Image layer model identity does not match its frozen spec".into());
            }
            Ok(*model)
        }
        ImageExecutionTarget::Remote { .. } => Err("Image layers run on local models".into()),
    }
}
