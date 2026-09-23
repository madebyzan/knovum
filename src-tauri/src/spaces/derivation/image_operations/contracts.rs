use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
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
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImageViewMode {
    Subject,
    Camera,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ImageZoom {
    Close,
    Medium,
    Wide,
}

impl ImageOperation {
    pub fn validate(&self) -> Result<(), String> {
        if let Self::MultiAngle { yaw, pitch, .. } = self {
            if !(-90..=90).contains(yaw) || !(-30..=60).contains(pitch) {
                return Err("Image viewing angle is out of range".into());
            }
        }
        Ok(())
    }

    pub fn view_prompt(&self) -> Option<String> {
        let Self::MultiAngle { mode, yaw, pitch, zoom } = self else {
            return None;
        };
        let horizontal = if *yaw >= 0 { "clockwise" } else { "counterclockwise" };
        let vertical = if *pitch >= 0 { "upward from" } else { "downward from" };
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

    fn view_key(&self) -> String {
        match self {
            Self::MultiAngle { mode, yaw, pitch, zoom } => {
                let mode = match mode {
                    ImageViewMode::Subject => "subject",
                    ImageViewMode::Camera => "camera",
                };
                let zoom = match zoom {
                    ImageZoom::Close => "close",
                    ImageZoom::Medium => "medium",
                    ImageZoom::Wide => "wide",
                };
                format!("{mode}:{yaw}:{pitch}:{zoom}")
            }
            Self::Upscale { .. } => String::new(),
        }
    }
}

impl ImageOperation {
    pub fn action_kind(&self) -> &'static str {
        match self {
            Self::Upscale { .. } => "upscale-image",
            Self::MultiAngle { .. } => "generate-image-angle",
        }
    }

    pub fn model(&self) -> &'static str {
        match self {
            Self::Upscale { .. } => "wanx2.1-imageedit",
            Self::MultiAngle { .. } => "qwen-image-3.0-pro",
        }
    }

    pub fn engine(&self) -> String {
        match self {
            Self::Upscale { target_long_edge } => {
                format!("dashscope:{}:target-{target_long_edge}", self.model())
            }
            Self::MultiAngle { .. } => {
                format!("dashscope:{}:{}", self.model(), self.view_key())
            }
        }
    }

    pub fn rationale(&self) -> &'static str {
        match self {
            Self::Upscale { .. } => "Image upscaled from source image",
            Self::MultiAngle { .. } => "View generated from source image",
        }
    }

    pub fn target_long_edge(&self) -> Option<u32> {
        match self {
            Self::Upscale { target_long_edge } => Some(*target_long_edge),
            Self::MultiAngle { .. } => None,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateImageOperationRequest {
    pub id: String,
    pub space_id: String,
    pub source_node_id: String,
    pub output_node_id: String,
    pub source_revision: String,
    pub source_edit: Value,
    pub operation: ImageOperation,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageOperationCapabilities {
    pub upscale: ImageOperationCapability,
    pub multi_angle: ImageOperationCapability,
}

#[derive(Clone, Copy, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ImageOperationCapability {
    Ready,
    Blocked { reason: ImageOperationBlockReason },
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageOperationBlockReason {
    ConfigurationIncomplete,
    ProviderUnsupported,
    CredentialUnavailable,
}
