use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const IMAGE_SEGMENTATION_RESOLUTION: u32 = 320;
pub const IMAGE_SEGMENTATION_INPUT_BYTES: u64 = 3 * 320 * 320 * 4;
pub const IMAGE_SEGMENTATION_MASK_BYTES: u64 = 320 * 320 * 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum TensorDataType {
    Float32LittleEndian,
    Uint8,
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
    /// Milliseconds of execution the worker may spend on this request. See
    /// [`crate::speech_wire::InferenceRequestIdentity::budget_ms`] for why budgets are relative.
    pub budget_ms: u64,
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
