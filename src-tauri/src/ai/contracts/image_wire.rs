pub use knovum_inference_contracts::image_wire::*;

pub(crate) use crate::local_resources::model_catalog::{
    IMAGE_SEGMENTATION_QUALITY_SPEC_ID, IMAGE_SEGMENTATION_SPEC_ID,
};

#[derive(Clone, Debug)]
pub struct ImageSegmentationExecutionIdentity {
    pub model_spec_id: &'static str,
    pub owner_kind: String,
    pub owner_id: String,
    pub attempt_id: String,
    pub attempt_epoch: u64,
    pub input_revision: String,
    pub deadline_ms: u64,
}
