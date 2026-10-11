#[derive(
    Clone,
    Copy,
    Debug,
    Deserialize,
    Serialize,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    schemars::JsonSchema,
    ts_rs::TS,
)]
#[serde(rename_all = "camelCase")]
pub enum ImageModelVariant {
    RealEsrganGeneralX4v3,
    RealEsrganX4plus,
    Migan,
    EfficientSamTi,
    EfficientSamS,
    DepthAnythingV2SmallUint8,
    DepthAnythingV2Small,
}

impl ImageModelVariant {
    pub const fn name(self) -> &'static str {
        match self {
            Self::RealEsrganGeneralX4v3 => "real-esrgan-general-x4v3",
            Self::RealEsrganX4plus => "real-esrgan-x4plus",
            Self::Migan => "mi-gan",
            Self::EfficientSamTi => "efficient-sam-ti",
            Self::EfficientSamS => "efficient-sam-s",
            Self::DepthAnythingV2SmallUint8 => "depth-anything-v2-small-uint8",
            Self::DepthAnythingV2Small => "depth-anything-v2-small-fp32",
        }
    }
}
impl ImageModelVariant {
    pub fn capability(self) -> RuntimeCapability {
        match self {
            Self::RealEsrganGeneralX4v3 | Self::RealEsrganX4plus => {
                RuntimeCapability::ImageUpscaling
            }
            Self::Migan => RuntimeCapability::ImageInpainting,
            Self::EfficientSamTi | Self::EfficientSamS => RuntimeCapability::ImageSelection,
            Self::DepthAnythingV2SmallUint8 | Self::DepthAnythingV2Small => {
                RuntimeCapability::ImageDepth
            }
        }
    }
}

use super::image_wire::TensorFileReference;
use super::runtime::RuntimeCapability;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageRuntimeModelPath {
    pub variant: ImageModelVariant,
    pub directory: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageSelectionPoint {
    pub x: f32,
    pub y: f32,
    pub foreground: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageSelectionRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ImageComputation {
    Upscale,
    Inpaint {
        #[serde(rename = "maskRef")]
        mask_ref: TensorFileReference,
    },
    Select {
        points: Vec<ImageSelectionPoint>,
        #[serde(rename = "boxRect")]
        box_rect: Option<ImageSelectionRect>,
    },
    Depth,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageComputeParams {
    pub request_id: String,
    pub owner_kind: String,
    pub owner_id: String,
    pub attempt_id: String,
    pub attempt_epoch: u64,
    pub input_revision: String,
    pub model_spec_id: String,
    pub deadline_ms: u64,
    pub tensor_ref: TensorFileReference,
    pub computation: ImageComputation,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageComputationOutput {
    pub request_id: String,
    pub attempt_epoch: u64,
    pub model_spec_id: String,
    pub tensor_ref: TensorFileReference,
}
