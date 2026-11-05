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
    Lama,
    EfficientSamTi,
    EfficientSamS,
    DepthAnythingV2SmallUint8,
    DepthAnythingV2Small,
    PpOcrV6Small,
}

impl ImageModelVariant {
    pub const fn name(self) -> &'static str {
        match self {
            Self::RealEsrganGeneralX4v3 => "real-esrgan-general-x4v3",
            Self::RealEsrganX4plus => "real-esrgan-x4plus",
            Self::Migan => "mi-gan",
            Self::Lama => "lama",
            Self::EfficientSamTi => "efficient-sam-ti",
            Self::EfficientSamS => "efficient-sam-s",
            Self::DepthAnythingV2SmallUint8 => "depth-anything-v2-small-uint8",
            Self::DepthAnythingV2Small => "depth-anything-v2-small-fp32",
            Self::PpOcrV6Small => "pp-ocrv6-small",
        }
    }
    pub fn capability(self) -> RuntimeCapability {
        match self {
            Self::RealEsrganGeneralX4v3 | Self::RealEsrganX4plus => {
                RuntimeCapability::ImageUpscaling
            }
            Self::Migan | Self::Lama => RuntimeCapability::ImageInpainting,
            Self::EfficientSamTi | Self::EfficientSamS => RuntimeCapability::ImageSelection,
            Self::DepthAnythingV2SmallUint8 | Self::DepthAnythingV2Small => {
                RuntimeCapability::ImageDepth
            }
            Self::PpOcrV6Small => RuntimeCapability::ImageTextRecognition,
        }
    }
}

use super::image_wire::{ImageSegmentParams, TensorFileReference};
use super::runtime::RuntimeCapability;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    /// Decomposes the source into disjoint object candidates. The worker encodes the source once,
    /// decodes one mask per seed, keeps the strongest non-overlapping candidates and returns a
    /// single label map whose pixel values are candidate ranks: `0` is unassigned and `1` is the
    /// highest ranked candidate. Ranking stays with the worker because only the decoder output
    /// carries the logits that stability scoring needs.
    DiscoverLayers {
        seeds: Vec<ImageSelectionPoint>,
        #[serde(rename = "maxLayers")]
        max_layers: u8,
    },
}

/// Seeds one discovery request may carry. The source embedding is encoded once and reused, so a
/// seed costs one decoder pass over a small probe rather than another encode. The ceiling is
/// therefore generous on purpose: a caller is expected to spend it on exploring the whole canvas,
/// because an object that is never prompted can never become a layer.
pub const MAX_DISCOVERY_SEEDS: usize = 64;
/// Candidates one discovery request may return. Ranks share a `u8` label map with the unassigned
/// `0`, which leaves ample headroom above this ceiling.
pub const MAX_DISCOVERED_LAYERS: u8 = 24;

/// Checks the bounds both sides of the worker boundary agree on. The host keeps its request inside
/// them and the worker rejects anything outside them.
pub fn validate_discovery(seeds: usize, max_layers: u8) -> Result<(), String> {
    if seeds == 0 || seeds > MAX_DISCOVERY_SEEDS {
        return Err(format!(
            "invalid_input: layer discovery takes 1 to {MAX_DISCOVERY_SEEDS} seeds"
        ));
    }
    if !(1..=MAX_DISCOVERED_LAYERS).contains(&max_layers) {
        return Err(format!(
            "invalid_input: layer discovery takes a budget of 1 to {MAX_DISCOVERED_LAYERS} layers"
        ));
    }
    Ok(())
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
    /// Milliseconds of execution the worker may spend on this request. See
    /// [`crate::speech_wire::InferenceRequestIdentity::budget_ms`] for why budgets are relative.
    pub budget_ms: u64,
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

/// Text recognition asks exactly what segmentation asks: one source tensor under one identity.
pub type ImageTextRecognitionParams = ImageSegmentParams;

/// One line of text, located in source pixels. Polygons are reduced to their axis-aligned bounds:
/// callers read and cite lines, and nothing downstream can use a finer shape.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecognizedTextLine {
    pub text: String,
    pub confidence: f32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageTextRecognition {
    pub request_id: String,
    pub attempt_epoch: u64,
    pub model_spec_id: String,
    pub lines: Vec<RecognizedTextLine>,
}
