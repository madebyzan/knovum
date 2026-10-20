use super::{ImageOperation, ImageOperationCapability};
use crate::{ai::tasks::TaskTarget, local_resources::model_catalog::LocalModelId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, JsonSchema, ts_rs::TS)]
#[serde(
    tag = "type",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum ImageExecutionTarget {
    Local {
        model: LocalModelId,
        spec_id: String,
    },
    Remote {
        target: TaskTarget,
    },
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct ImageOperationTargetOption {
    pub execution: ImageExecutionTarget,
    pub label: String,
    pub capability: crate::local_resources::model_catalog::LocalModelCapability,
    pub availability: ImageOperationCapability,
    pub selected: bool,
    pub maximum_side: Option<u32>,
    pub maximum_pixels: Option<u64>,
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageTransformParameters {
    #[schemars(length(min = 1, max = 512))]
    pub title: String,
    pub operation: ImageOperation,
    pub execution: ImageExecutionTarget,
}

#[derive(Clone, Deserialize, Serialize, JsonSchema, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct ImageSelection {
    pub points: Vec<crate::ai::contracts::image_compute::ImageSelectionPoint>,
    pub box_rect: Option<crate::ai::contracts::image_compute::ImageSelectionRect>,
    pub strokes: Vec<super::MaskStroke>,
}

impl ImageSelection {
    pub(crate) fn validate(&self) -> Result<(), String> {
        let capacity = self.points.len() + if self.box_rect.is_some() { 2 } else { 0 };
        if capacity == 0
            || capacity > 6
            || (self.box_rect.is_none() && !self.points.iter().any(|point| point.foreground))
            || self.points.iter().any(|point| {
                !point.x.is_finite()
                    || !point.y.is_finite()
                    || !(0.0..=1.0).contains(&point.x)
                    || !(0.0..=1.0).contains(&point.y)
            })
        {
            return Err(
                "Selection requires up to six normalized prompts and a foreground point or box"
                    .into(),
            );
        }
        if let Some(rect) = &self.box_rect {
            if [rect.x, rect.y, rect.width, rect.height]
                .iter()
                .any(|value| !value.is_finite())
                || rect.x < 0.0
                || rect.y < 0.0
                || rect.width <= 0.0
                || rect.height <= 0.0
                || rect.x + rect.width > 1.0
                || rect.y + rect.height > 1.0
            {
                return Err("Selection box lies outside the rendered image".into());
            }
        }
        if !self.strokes.is_empty() {
            ImageOperation::RemoveObject {
                strokes: self.strokes.clone(),
            }
            .validate()?;
        }
        Ok(())
    }
}

impl ImageExecutionTarget {
    pub(crate) fn validate(&self, operation: &ImageOperation) -> Result<(), String> {
        if matches!(self, Self::Remote { .. })
            && super::super::routing::local_capability(operation).is_some()
        {
            return Err("This image operation requires a local model".into());
        }
        if let Self::Local { model, spec_id } = self {
            let capability = super::super::routing::local_capability(operation);
            if capability != Some(model.capability()) || *spec_id != model.image_spec()? {
                return Err("Image model identity does not support the frozen operation".into());
            }
        }
        Ok(())
    }
}
