use tensor::{encode_f32, read_mask, stage_tensor, valid_sha256, validate_input};
pub(crate) mod compute;
mod session;
mod tensor;

use super::lease::RequestIdentity;
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use crate::{
    ai::contracts::{
        image_wire::{
            ImageSegmentParams, ImageSegmentation, ImageSegmentationExecutionIdentity,
            TensorDataType, TensorFileReference, IMAGE_SEGMENTATION_INPUT_BYTES,
            IMAGE_SEGMENTATION_MASK_BYTES, IMAGE_SEGMENTATION_QUALITY_SPEC_ID,
            IMAGE_SEGMENTATION_RESOLUTION, IMAGE_SEGMENTATION_SPEC_ID,
        },
        runtime::{RuntimeCapability, RuntimeModel},
    },
    local_resources::{models::catalog::ModelKind, preparation::PreparedModels},
};

const INPUT_SHAPE: [u32; 4] = [
    1,
    3,
    IMAGE_SEGMENTATION_RESOLUTION,
    IMAGE_SEGMENTATION_RESOLUTION,
];
const MASK_SHAPE: [u32; 4] = [
    1,
    1,
    IMAGE_SEGMENTATION_RESOLUTION,
    IMAGE_SEGMENTATION_RESOLUTION,
];

pub(crate) async fn segment_image<R: Runtime>(
    app: &AppHandle<R>,
    input: Vec<f32>,
    identity: &ImageSegmentationExecutionIdentity,
    resources: &PreparedModels,
    cancellation: &CancellationToken,
) -> Result<Vec<f32>, String> {
    validate_input(&input)?;
    let model_spec_id = identity.model_spec_id;
    let model = match model_spec_id {
        IMAGE_SEGMENTATION_SPEC_ID => ModelKind::ImageSegmentation,
        IMAGE_SEGMENTATION_QUALITY_SPEC_ID => ModelKind::ImageSegmentationQuality,
        _ => return Err("unsupported_model: image segmentation identity is unavailable".into()),
    };
    let model_path = resources.path(model)?;
    let mut lease = session::acquire(
        app,
        RuntimeCapability::ImageSegmentation,
        model_spec_id,
        RuntimeModel::Snapshot {
            path: model_path.to_string_lossy().into_owned(),
        },
        cancellation,
    )
    .await?;
    let request_id = uuid::Uuid::new_v4().to_string();
    let work_directory = lease.work_directory()?.to_owned();
    let tensor_name = format!("tensor-{request_id}.f32");
    let tensor_path = work_directory.join(&tensor_name);
    let tensor_bytes = encode_f32(&input);
    let tensor_sha256 = hex::encode(Sha256::digest(&tensor_bytes));
    stage_tensor(&tensor_path, &tensor_bytes).await?;
    let params = ImageSegmentParams {
        request_id: request_id.clone(),
        owner_kind: identity.owner_kind.clone(),
        owner_id: identity.owner_id.clone(),
        attempt_id: identity.attempt_id.clone(),
        attempt_epoch: identity.attempt_epoch,
        input_revision: identity.input_revision.clone(),
        model_spec_id: model_spec_id.into(),
        budget_ms: identity.budget_ms,
        tensor_ref: TensorFileReference {
            file_name: tensor_name,
            byte_length: IMAGE_SEGMENTATION_INPUT_BYTES,
            sha256: tensor_sha256,
            shape: INPUT_SHAPE,
            data_type: TensorDataType::Float32LittleEndian,
        },
    };
    let segmentation = lease
        .request::<ImageSegmentation>(
            &RequestIdentity {
                request_id: &request_id,
                attempt_epoch: identity.attempt_epoch,
                budget_ms: identity.budget_ms,
            },
            &params,
            cancellation,
        )
        .await;
    let _ = tokio::fs::remove_file(&tensor_path).await;
    let output = segmentation?;
    let expected_mask = format!("mask-{request_id}.f32");
    if output.request_id != request_id
        || output.attempt_epoch != identity.attempt_epoch
        || output.model_spec_id != model_spec_id
        || output.mask_ref.file_name != expected_mask
        || output.mask_ref.byte_length != IMAGE_SEGMENTATION_MASK_BYTES
        || !valid_sha256(&output.mask_ref.sha256)
        || output.mask_ref.shape != MASK_SHAPE
        || !matches!(
            output.mask_ref.data_type,
            TensorDataType::Float32LittleEndian
        )
    {
        lease.discard().await;
        return Err("invalid_output: image segmentation response identity differs".into());
    }
    let mask_path = work_directory.join(&expected_mask);
    let mask_sha256 = output.mask_ref.sha256;
    let reading = {
        let path = mask_path.clone();
        crate::foundation::blocking::run(move || read_mask(&path, &mask_sha256)).await
    };
    let _ = tokio::fs::remove_file(&mask_path).await;
    match reading {
        Ok(mask) => Ok(mask),
        Err(error) => {
            lease.discard().await;
            Err(error)
        }
    }
}
