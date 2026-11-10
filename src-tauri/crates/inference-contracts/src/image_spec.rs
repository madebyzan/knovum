//! Frozen specifications for local image models. Defines input scaling, channel standardization,
//! geometry transformation, resampling filter, output head selection, and output field semantics
//! as a single neutral source of truth shared by both host and worker crates.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// How pixel values map to numerical tensor inputs.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Scale {
    /// Divided by 255.0: maps `[0, 255]` to `[0.0, 1.0]`.
    UnitScale,
    /// Divided by `max(np.max(image), min_scale)`: adaptive whole-image scalar normalization.
    AdaptiveMaxScale { min_scale: f32 },
    /// Raw `[0, 255]` pixel values passed through unscaled. (MobileSAM: the encoder bakes its own
    /// padding and normalization.)
    Identity,
}

/// Channel-wise affine standardization.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Standardization {
    /// No standardization applied.
    None,
    /// Standard ImageNet mean and standard deviation.
    ImageNet { mean: [f32; 3], std: [f32; 3] },
}

/// Geometry transformation from source image to model input tensor.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Geometry {
    /// Stretched to fixed `width` × `height`, ignoring aspect ratio. (Segmentation: 1024×1024,
    /// SAM 2.1: 1024×1024.)
    Stretch { width: u32, height: u32 },
    /// Resized such that the longest edge does not exceed `edge`. (MobileSAM: 1024.)
    LongestEdge { edge: u32 },
    /// Resized maintaining aspect ratio, with the shortest edge target and aligned to `patch`. (Depth: 518 / 14).
    ShortestEdgePatchAligned { edge: u32, patch: u32 },
    /// Tiled execution with core and context overlap. (Real-ESRGAN: core 64, context 32, tile 128).
    Tiled { core: u32, context: u32 },
    /// Fitted window around the target hole. (Big LaMa: 512, context share 0.5, min context 32).
    FittedWindow {
        edge: u32,
        context_share: f32,
        min_context: u32,
    },
}

/// Neutral resampling filter representation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum Resample {
    Lanczos3,
    CatmullRom,
    Triangle,
    Nearest,
}

/// Selection of output head from the ONNX graph outputs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OutputHead {
    /// Single output tensor `[1, 1, H, W]` or `[1, 3, H, W]`.
    Single,
    /// First head of multi-supervision outputs (`ort_outs[0][:, 0, :, :]`).
    FirstOfSupervision,
    /// Selection candidate heads ranked by score (e.g. SAM 2.1: 3 candidate heads).
    BestOfScored { count: usize },
}

/// Normalization applied to scalar fields.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum OutputNormalization {
    /// Min-max normalized across all elements to `[0.0, 1.0]`.
    MinMax,
}

/// Semantics of the model's output tensor field.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OutputField {
    /// Raw scalar field requiring post-processing normalization. (Depth, segmentation.)
    RawScalarField { normalize: OutputNormalization },
    /// Already in normalized probability `[0.0, 1.0]`.
    ProbabilityFullRange,
    /// Float RGB in `[0.0, 1.0]` range quantized to uint8. (Real-ESRGAN).
    UnitRgb,
    /// Float RGB in `[0.0, scale]` range quantized to uint8. (Big LaMa with scale = 255.0).
    ScaledRgb { scale: f32 },
    /// Logits evaluated relative to decision threshold offset. (MobileSAM and SAM 2.1 with
    /// offset = 0.0.)
    Logits { decision_offset: f32 },
}

/// Provenance metadata tracking the origin, reference implementation, and revision.
#[derive(Clone, Copy, Debug, PartialEq, Eq, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct Provenance {
    pub reference: &'static str,
    pub revision: &'static str,
    pub shipped_processor: Option<&'static str>,
}

/// Complete frozen inference specification for an image model.
#[derive(Clone, Copy, Debug, PartialEq, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct ImageModelSpec {
    pub scale: Scale,
    pub standardization: Standardization,
    pub geometry: Geometry,
    pub resample: Resample,
    pub output_head: OutputHead,
    pub output_field: OutputField,
    pub provenance: Provenance,
}

pub const HIGH_RES_SEGMENTATION_RESOLUTION: u32 = 1024;

/// Frozen specification for BiRefNet high-resolution bilateral reference segmentation (`birefnet-general-lite`).
pub const BIREFNET_SPEC: ImageModelSpec = ImageModelSpec {
    scale: Scale::UnitScale,
    standardization: Standardization::ImageNet {
        mean: [0.485, 0.456, 0.406],
        std: [0.229, 0.224, 0.225],
    },
    geometry: Geometry::Stretch {
        width: HIGH_RES_SEGMENTATION_RESOLUTION,
        height: HIGH_RES_SEGMENTATION_RESOLUTION,
    },
    resample: Resample::Triangle,
    output_head: OutputHead::Single,
    output_field: OutputField::RawScalarField {
        normalize: OutputNormalization::MinMax,
    },
    provenance: Provenance {
        reference: "https://huggingface.co/onnx-community/BiRefNet_lite-ONNX",
        revision: "de15b22ba131738a16dff04aab8bdf8dc32e3ac1",
        shipped_processor: Some("preprocessor_config.json"),
    },
};

/// Frozen specification for RMBG-2.0 state-of-the-art background removal (`briaai/RMBG-2.0`).
pub const RMBG20_SPEC: ImageModelSpec = ImageModelSpec {
    scale: Scale::UnitScale,
    standardization: Standardization::ImageNet {
        mean: [0.485, 0.456, 0.406],
        std: [0.229, 0.224, 0.225],
    },
    geometry: Geometry::Stretch {
        width: HIGH_RES_SEGMENTATION_RESOLUTION,
        height: HIGH_RES_SEGMENTATION_RESOLUTION,
    },
    resample: Resample::Triangle,
    output_head: OutputHead::Single,
    output_field: OutputField::RawScalarField {
        normalize: OutputNormalization::MinMax,
    },
    provenance: Provenance {
        reference: "https://huggingface.co/briaai/RMBG-2.0",
        revision: "fcd7830ebd0cc92c5e6262da225a7cc41f2a9afd",
        shipped_processor: Some("preprocessor_config.json"),
    },
};

/// Frozen specification for BiRefNet Portrait fine human hair and fur segmentation (`birefnet-portrait`).
pub const BIREFNET_PORTRAIT_SPEC: ImageModelSpec = ImageModelSpec {
    scale: Scale::UnitScale,
    standardization: Standardization::ImageNet {
        mean: [0.485, 0.456, 0.406],
        std: [0.229, 0.224, 0.225],
    },
    geometry: Geometry::Stretch {
        width: HIGH_RES_SEGMENTATION_RESOLUTION,
        height: HIGH_RES_SEGMENTATION_RESOLUTION,
    },
    resample: Resample::Triangle,
    output_head: OutputHead::Single,
    output_field: OutputField::RawScalarField {
        normalize: OutputNormalization::MinMax,
    },
    provenance: Provenance {
        reference: "https://huggingface.co/onnx-community/BiRefNet-portrait-ONNX",
        revision: "dd7167f6a8b54ff7efc29a4c988938d79866464f",
        shipped_processor: Some("preprocessor_config.json"),
    },
};

/// Frozen specification for RMBG-1.4 high-resolution background removal (`briaai/RMBG-1.4`).
pub const RMBG14_SPEC: ImageModelSpec = ImageModelSpec {
    scale: Scale::UnitScale,
    standardization: Standardization::ImageNet {
        mean: [0.5, 0.5, 0.5],
        std: [1.0, 1.0, 1.0],
    },
    geometry: Geometry::Stretch {
        width: HIGH_RES_SEGMENTATION_RESOLUTION,
        height: HIGH_RES_SEGMENTATION_RESOLUTION,
    },
    resample: Resample::Lanczos3,
    output_head: OutputHead::Single,
    output_field: OutputField::RawScalarField {
        normalize: OutputNormalization::MinMax,
    },
    provenance: Provenance {
        reference: "https://huggingface.co/briaai/RMBG-1.4",
        revision: "2ceba5a5efaec153162aedea169f76caf9b46cf8",
        shipped_processor: Some("preprocessor_config.json"),
    },
};

/// Frozen specification for IS-Net / DIS high-resolution segmentation (`isnet-general-use`).
pub const ISNET_DIS_SPEC: ImageModelSpec = ImageModelSpec {
    scale: Scale::AdaptiveMaxScale { min_scale: 1e-6 },
    standardization: Standardization::ImageNet {
        mean: [0.5, 0.5, 0.5],
        std: [1.0, 1.0, 1.0],
    },
    geometry: Geometry::Stretch {
        width: HIGH_RES_SEGMENTATION_RESOLUTION,
        height: HIGH_RES_SEGMENTATION_RESOLUTION,
    },
    resample: Resample::Lanczos3,
    output_head: OutputHead::FirstOfSupervision,
    output_field: OutputField::RawScalarField {
        normalize: OutputNormalization::MinMax,
    },
    provenance: Provenance {
        reference: "https://github.com/danielgatis/rembg",
        revision: "cd3a3d6767a7859efea31ef0f2f373582cf06d82",
        shipped_processor: None,
    },
};

/// Frozen specification for IS-Net Anime high-resolution segmentation (`isnet-anime`).
pub const ISNET_ANIME_SPEC: ImageModelSpec = ImageModelSpec {
    scale: Scale::AdaptiveMaxScale { min_scale: 1e-6 },
    standardization: Standardization::ImageNet {
        mean: [0.5, 0.5, 0.5],
        std: [1.0, 1.0, 1.0],
    },
    geometry: Geometry::Stretch {
        width: HIGH_RES_SEGMENTATION_RESOLUTION,
        height: HIGH_RES_SEGMENTATION_RESOLUTION,
    },
    resample: Resample::Lanczos3,
    output_head: OutputHead::FirstOfSupervision,
    output_field: OutputField::RawScalarField {
        normalize: OutputNormalization::MinMax,
    },
    provenance: Provenance {
        reference: "https://github.com/danielgatis/rembg",
        revision: "cd3a3d6767a7859efea31ef0f2f373582cf06d82",
        shipped_processor: None,
    },
};

/// Frozen specification for Depth Anything V2 Small (`depth-anything-v2-small-fp32` / `uint8`).
pub const DEPTH_ANYTHING_V2_SPEC: ImageModelSpec = ImageModelSpec {
    scale: Scale::UnitScale,
    standardization: Standardization::ImageNet {
        mean: [0.485, 0.456, 0.406],
        std: [0.229, 0.224, 0.225],
    },
    geometry: Geometry::ShortestEdgePatchAligned {
        edge: 518,
        patch: 14,
    },
    resample: Resample::CatmullRom,
    output_head: OutputHead::Single,
    output_field: OutputField::RawScalarField {
        normalize: OutputNormalization::MinMax,
    },
    provenance: Provenance {
        reference: "https://github.com/DepthAnything/Depth-Anything-V2",
        revision: "4472b7362082ad9968fee890ca0f1e5aca36b93d",
        shipped_processor: Some("preprocessor_config.json"),
    },
};

/// Frozen specification for Real-ESRGAN upscaling models (`x4plus`, `general-x4v3`).
pub const REAL_ESRGAN_SPEC: ImageModelSpec = ImageModelSpec {
    scale: Scale::UnitScale,
    standardization: Standardization::None,
    geometry: Geometry::Tiled {
        core: 64,
        context: 32,
    },
    resample: Resample::Nearest,
    output_head: OutputHead::Single,
    output_field: OutputField::UnitRgb,
    provenance: Provenance {
        reference: "https://github.com/xinntao/Real-ESRGAN",
        revision: "qai-hub-v0.62.2",
        shipped_processor: None,
    },
};

/// Frozen specification for MobileSAM interactive selection (`mobile_sam_20230629`). The encoder
/// takes raw `[0, 255]` HWC pixels resized so the longest edge is 1024 and bakes its own padding
/// and normalization; the decoder emits single-mask logits at 256×256 with a predicted IoU.
pub const MOBILE_SAM_SPEC: ImageModelSpec = ImageModelSpec {
    scale: Scale::Identity,
    standardization: Standardization::None,
    geometry: Geometry::LongestEdge { edge: 1024 },
    resample: Resample::Triangle,
    output_head: OutputHead::Single,
    output_field: OutputField::Logits {
        decision_offset: 0.0,
    },
    provenance: Provenance {
        reference: "https://huggingface.co/vietanhdev/segment-anything-onnx-models",
        revision: "9effc01a9e135621d710d49159f1ffb0b6f724dc",
        shipped_processor: None,
    },
};

/// Frozen specification for SAM 2.1 Hiera-Tiny interactive selection (`sam2.1_hiera_tiny_20260221`).
/// The encoder stretches the source to 1024×1024 and expects unit-scaled ImageNet-standardized
/// CHW pixels; the decoder emits three candidate mask logits at 256×256 with predicted IoUs.
pub const SAM2_HIERA_TINY_SPEC: ImageModelSpec = ImageModelSpec {
    scale: Scale::UnitScale,
    standardization: Standardization::ImageNet {
        mean: [0.485, 0.456, 0.406],
        std: [0.229, 0.224, 0.225],
    },
    geometry: Geometry::Stretch {
        width: 1024,
        height: 1024,
    },
    resample: Resample::Triangle,
    output_head: OutputHead::BestOfScored { count: 3 },
    output_field: OutputField::Logits {
        decision_offset: 0.0,
    },
    provenance: Provenance {
        reference: "https://huggingface.co/vietanhdev/segment-anything-2.1-onnx-models",
        revision: "6a3ac868340a3196a349050a6efae22a5acc0330",
        shipped_processor: None,
    },
};

/// Frozen specification for Big LaMa mask repair (`lama_fp32.onnx`).
pub const LAMA_SPEC: ImageModelSpec = ImageModelSpec {
    scale: Scale::UnitScale,
    standardization: Standardization::None,
    geometry: Geometry::FittedWindow {
        edge: 512,
        context_share: 0.5,
        min_context: 32,
    },
    resample: Resample::Triangle,
    output_head: OutputHead::Single,
    output_field: OutputField::ScaledRgb { scale: 255.0 },
    provenance: Provenance {
        reference: "https://github.com/advimman/lama",
        revision: "c3c0c9e468934d62e79c329e35d82dd09ff8c444",
        shipped_processor: None,
    },
};
