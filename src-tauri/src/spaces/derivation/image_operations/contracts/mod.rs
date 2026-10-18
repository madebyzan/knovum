mod target;
use serde::{Deserialize, Serialize};
use serde_json::Value;
pub use target::{
    ImageExecutionTarget, ImageOperationTargetOption, ImageSelection, ImageTransformParameters,
};

#[derive(Clone, Deserialize, Serialize, schemars::JsonSchema, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum ImageOperation {
    Upscale {
        #[serde(rename = "targetLongEdge")]
        target_long_edge: u32,
    },
    MultiAngle {
        mode: ImageViewMode,
        yaw: i16,
        pitch: i16,
        zoom: ImageZoom,
    },
    RemoveObject {
        strokes: Vec<MaskStroke>,
    },
    RemoveBackground,
    ExtractSelection {
        selection: ImageSelection,
    },
    RemoveSelectedObject {
        selection: ImageSelection,
        #[serde(rename = "selectionModel")]
        selection_model: crate::local_resources::model_catalog::LocalModelId,
        #[serde(rename = "selectionSpecId")]
        selection_spec_id: String,
    },
    GenerateDepth,
    RemoveObjectPrompt {
        prompt: String,
    },
}

#[derive(Clone, Deserialize, Serialize, schemars::JsonSchema, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct MaskStroke {
    pub diameter: f64,
    pub points: Vec<f64>,
}

#[derive(Clone, Copy, Deserialize, Serialize, schemars::JsonSchema, ts_rs::TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum ImageViewMode {
    Subject,
    Camera,
}

#[derive(Clone, Copy, Deserialize, Serialize, schemars::JsonSchema, ts_rs::TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum ImageZoom {
    Close,
    Medium,
    Wide,
}

impl ImageOperation {
    pub fn validate(&self) -> Result<(), String> {
        if let Self::Upscale { target_long_edge } = self {
            if !matches!(target_long_edge, 2048 | 4096 | 8192) {
                return Err("Unsupported upscale target".into());
            }
        }
        if let Self::RemoveSelectedObject {
            selection_model,
            selection_spec_id,
            ..
        } = self
        {
            if selection_model.capability()
                != crate::local_resources::model_catalog::LocalModelCapability::ImageSelection
                || selection_model.image_spec()? != *selection_spec_id
            {
                return Err("Invalid selection model identity".into());
            }
        }
        if let Self::ExtractSelection { selection } | Self::RemoveSelectedObject { selection, .. } =
            self
        {
            selection.validate()?;
        }
        if let Self::RemoveObject { strokes } = self {
            if strokes.is_empty() || strokes.len() > 100 {
                return Err("Object removal requires 1–100 mask strokes".into());
            }
            let mut total_points = 0;
            for stroke in strokes {
                if !stroke.diameter.is_finite() || !(0.005..=0.15).contains(&stroke.diameter) {
                    return Err("Object removal brush size is invalid".into());
                }
                if stroke.points.len() < 4 || stroke.points.len() % 2 != 0 {
                    return Err("Object removal mask stroke is invalid".into());
                }
                total_points += stroke.points.len() / 2;
                if total_points > 10_000
                    || stroke
                        .points
                        .iter()
                        .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
                {
                    return Err("Object removal mask points are invalid".into());
                }
            }
        }
        if let Self::RemoveObjectPrompt { prompt } = self {
            if prompt.trim().is_empty() || prompt.chars().count() > 1000 {
                return Err("Object removal description must contain 1–1000 characters".into());
            }
        }
        if let Self::MultiAngle { yaw, pitch, .. } = self {
            if !(-90..=90).contains(yaw) || !(-30..=60).contains(pitch) {
                return Err("Image viewing angle is out of range".into());
            }
        }
        Ok(())
    }

    pub fn view_prompt(&self) -> Option<String> {
        let Self::MultiAngle {
            mode,
            yaw,
            pitch,
            zoom,
        } = self
        else {
            return None;
        };
        let horizontal = if *yaw >= 0 {
            "clockwise"
        } else {
            "counterclockwise"
        };
        let vertical = if *pitch >= 0 {
            "upward from"
        } else {
            "downward from"
        };
        let orientation = match mode {
            ImageViewMode::Subject => format!(
                "Rotate the subject {} degrees {horizontal} around its vertical axis and tilt it {} degrees {vertical} its original orientation while the camera remains fixed.",
                yaw.abs(), pitch.abs()
            ),
            ImageViewMode::Camera => format!(
                "Move the camera {} degrees {horizontal} around the subject and {} degrees {vertical} its original position while keeping the subject fixed.",
                yaw.abs(), pitch.abs()
            ),
        };
        let framing = match zoom {
            ImageZoom::Close => "Frame a close view of the subject.",
            ImageZoom::Medium => "Keep the original medium framing.",
            ImageZoom::Wide => "Frame a wider view with more surrounding space.",
        };
        Some(format!(
            "Create one new view from the input image. {orientation} {framing} Preserve subject identity, clothing, materials, proportions, and visual style."
        ))
    }
}

impl ImageOperation {
    pub fn action_kind(&self) -> &'static str {
        match self {
            Self::Upscale { .. } => "upscale-image",
            Self::RemoveBackground => "remove-image-background",
            Self::ExtractSelection { .. } => "extract-image-selection",
            Self::GenerateDepth => "generate-image-depth",
            Self::MultiAngle { .. } => "generate-image-angle",
            Self::RemoveObject { .. }
            | Self::RemoveObjectPrompt { .. }
            | Self::RemoveSelectedObject { .. } => "remove-image-object",
        }
    }

    pub fn engine(&self, provider: &str, configured_model: &str) -> String {
        let model = match (provider, self) {
            ("qwen", Self::Upscale { .. } | Self::RemoveObject { .. }) => "wanx2.1-imageedit",
            ("qwen", Self::MultiAngle { .. }) => "qwen-image-3.0-pro",
            _ => configured_model,
        };
        let provider = if provider == "qwen" {
            "dashscope"
        } else {
            provider
        };
        match self {
            Self::Upscale { target_long_edge } => {
                format!("{provider}:{model}:target-{target_long_edge}")
            }
            Self::MultiAngle {
                mode,
                yaw,
                pitch,
                zoom,
            } => {
                let mode = match mode {
                    ImageViewMode::Subject => "subject",
                    ImageViewMode::Camera => "camera",
                };
                let zoom = match zoom {
                    ImageZoom::Close => "close",
                    ImageZoom::Medium => "medium",
                    ImageZoom::Wide => "wide",
                };
                format!("{provider}:{model}:{mode}:{yaw}:{pitch}:{zoom}")
            }
            Self::RemoveBackground | Self::ExtractSelection { .. } | Self::GenerateDepth => {
                format!("{provider}:{model}")
            }
            Self::RemoveObject { .. } | Self::RemoveSelectedObject { .. } => {
                format!("{provider}:{model}:mask")
            }
            Self::RemoveObjectPrompt { .. } => format!("{provider}:{model}:prompt"),
        }
    }
}

#[derive(Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct DeriveImageRequest {
    pub id: String,
    pub space_id: String,
    pub source_node_id: String,
    pub output_node_id: String,
    pub source_revision: String,
    pub source_edit: Value,
    pub operation: ImageOperation,
    pub execution: ImageExecutionTarget,
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct ImageOperationCapabilities {
    pub targets: Vec<ImageOperationTargetOption>,
    pub remote: Option<ImageExecutionTarget>,
    pub selection: ImageOperationCapability,
    pub depth: ImageOperationCapability,
    pub upscale: ImageOperationCapability,
    pub multi_angle: ImageOperationCapability,
    pub masked_removal: ImageOperationCapability,
    pub prompt_removal: ImageOperationCapability,
}

#[derive(Clone, Copy, Serialize, ts_rs::TS)]
#[serde(tag = "state", rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum ImageOperationCapability {
    Ready,
    Unverified,
    Blocked { reason: ImageOperationBlockReason },
}

#[derive(Clone, Copy, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum ImageOperationBlockReason {
    ConfigurationIncomplete,
    ProviderUnsupported,
    CredentialUnavailable,
}

#[cfg(test)]
mod tests;
