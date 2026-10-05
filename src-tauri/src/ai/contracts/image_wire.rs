use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const IMAGE_SEGMENTATION_SPEC_ID: &str =
    "u2netp-onnx:309c8469258dda742793dce0ebea8e6dd393174f89934733ecc8b14c76f4ddd8-v1";
pub const IMAGE_SEGMENTATION_RESOLUTION: u32 = 320;
pub const IMAGE_SEGMENTATION_INPUT_BYTES: u64 = 3 * 320 * 320 * 4;
pub const IMAGE_SEGMENTATION_MASK_BYTES: u64 = 320 * 320 * 4;

#[derive(Clone, Debug)]
pub struct ImageSegmentationExecutionIdentity {
    pub owner_kind: String,
    pub owner_id: String,
    pub attempt_id: String,
    pub attempt_epoch: u64,
    pub input_revision: String,
    pub deadline_ms: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum TensorDataType {
    Float32LittleEndian,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TensorFileReference {
    pub file_name: String,
    pub byte_length: u64,
    pub sha256: String,
    pub shape: [u32; 4],
    pub data_type: TensorDataType,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageSegmentParams {
    pub request_id: String,
    pub owner_kind: String,
    pub owner_id: String,
    pub attempt_id: String,
    pub attempt_epoch: u64,
    pub input_revision: String,
    pub model_spec_id: String,
    pub deadline_ms: u64,
    pub tensor_ref: TensorFileReference,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ImageSegmentation {
    pub request_id: String,
    pub attempt_epoch: u64,
    pub model_spec_id: String,
    pub mask_ref: TensorFileReference,
}
