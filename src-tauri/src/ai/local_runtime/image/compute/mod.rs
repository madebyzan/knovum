use super::{session, tensor};
use crate::{
    ai::contracts::{
        image_compute::{
            ImageComputation, ImageComputationOutput, ImageComputeParams, ImageRuntimeModelPath,
            ImageSelectionPoint, ImageSelectionRect,
        },
        image_wire::{TensorDataType, TensorFileReference},
        runtime::{RuntimeCapability, RuntimeModelPaths},
    },
    local_resources::{models::catalog::ImageModelVariant, preparation::PreparedModels},
};
use sha2::{Digest, Sha256};
use std::io::Read;
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

pub(crate) struct ImageTensor {
    pub bytes: Vec<u8>,
    pub shape: [u32; 4],
    pub data_type: TensorDataType,
}

pub(crate) enum ImageTensorOperation {
    Upscale,
    Inpaint(ImageTensor),
    Select {
        points: Vec<ImageSelectionPoint>,
        box_rect: Option<ImageSelectionRect>,
    },
    Depth,
}

pub(crate) struct ImageComputeRequest {
    pub model: ImageModelVariant,
    pub owner_id: String,
    pub input_revision: String,
    pub deadline_ms: u64,
    pub input: ImageTensor,
    pub operation: ImageTensorOperation,
}

impl ImageModelVariant {
    pub(crate) fn capability(self) -> RuntimeCapability {
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

pub(crate) async fn execute<R: Runtime>(
    app: &AppHandle<R>,
    request: ImageComputeRequest,
    resources: &PreparedModels,
    cancellation: &CancellationToken,
) -> Result<ImageTensor, String> {
    let capability = match &request.operation {
        ImageTensorOperation::Upscale => RuntimeCapability::ImageUpscaling,
        ImageTensorOperation::Inpaint(_) => RuntimeCapability::ImageInpainting,
        ImageTensorOperation::Select { .. } => RuntimeCapability::ImageSelection,
        ImageTensorOperation::Depth => RuntimeCapability::ImageDepth,
    };
    if request.model.capability() != capability {
        return Err("invalid_input: model does not implement this image operation".into());
    }
    let shape = request.input.shape;
    let expected = output_shape(&request.operation, shape)?;
    let directory = resources.directory(request.model.models())?;
    let spec_id = request.model.spec_id();
    let mut lease = session::acquire(
        app,
        session::ImageWorkerConfiguration {
            model_path: &directory,
            paths: RuntimeModelPaths {
                image_processing: Some(ImageRuntimeModelPath {
                    variant: request.model,
                    directory: directory.to_string_lossy().into_owned(),
                }),
                ..Default::default()
            },
            capability: request.model.capability(),
            spec_id: &spec_id,
            deadline_ms: request.deadline_ms,
        },
        cancellation,
    )
    .await?;
    let worker = lease.guard.as_mut().ok_or("worker_crashed")?;
    let request_id = uuid::Uuid::new_v4().to_string();
    let work = worker.work_directory().to_owned();
    let source_name = format!("source-{request_id}.tensor");
    let source_ref = stage(&work, &source_name, request.input).await?;
    let mask_name = format!("selection-{request_id}.tensor");
    let computation = match request.operation {
        ImageTensorOperation::Upscale => ImageComputation::Upscale,
        ImageTensorOperation::Inpaint(mask) => {
            let staged = stage(&work, &mask_name, mask).await;
            if staged.is_err() {
                let _ = tokio::fs::remove_file(work.join(&source_name)).await;
            }
            ImageComputation::Inpaint { mask_ref: staged? }
        }
        ImageTensorOperation::Select { points, box_rect } => {
            ImageComputation::Select { points, box_rect }
        }
        ImageTensorOperation::Depth => ImageComputation::Depth,
    };
    let params = ImageComputeParams {
        request_id: request_id.clone(),
        owner_kind: "imageOperation".into(),
        owner_id: request.owner_id.clone(),
        attempt_id: request.owner_id,
        attempt_epoch: 1,
        input_revision: request.input_revision,
        model_spec_id: spec_id.clone(),
        deadline_ms: request.deadline_ms,
        tensor_ref: source_ref,
        computation,
    };
    let response = session::request::<_, ImageComputationOutput>(
        worker,
        session::ImageRequestIdentity {
            request_id: &request_id,
            attempt_epoch: 1,
            deadline_ms: request.deadline_ms,
        },
        &params,
        cancellation,
    )
    .await;
    let _ = tokio::fs::remove_file(work.join(source_name)).await;
    let _ = tokio::fs::remove_file(work.join(mask_name)).await;
    let validated = async {
        let response = response?;
        if response.request_id != request_id
            || response.attempt_epoch != 1
            || response.model_spec_id != spec_id
            || response.tensor_ref.file_name != format!("image-{request_id}.tensor")
            || response.tensor_ref.shape != expected.0
            || response.tensor_ref.data_type != expected.1
        {
            return Err("invalid_output: image response identity or tensor shape differs".into());
        }
        let path = work.join(&response.tensor_ref.file_name);
        let reference = response.tensor_ref;
        crate::blocking::run(move || read_output(&path, reference)).await
    }
    .await;
    let _ = tokio::fs::remove_file(work.join(format!("image-{request_id}.tensor"))).await;
    if validated.is_err() {
        worker.terminate().await;
        *lease.guard = None;
    }
    validated
}

async fn stage(
    directory: &std::path::Path,
    name: &str,
    tensor: ImageTensor,
) -> Result<TensorFileReference, String> {
    if tensor.bytes.len() as u64 != byte_length(tensor.shape, tensor.data_type)?
        || tensor.bytes.len() > 64 * 1024 * 1024
    {
        return Err("invalid_input: image tensor exceeds the admitted byte budget".into());
    }
    let reference = TensorFileReference {
        file_name: name.into(),
        byte_length: tensor.bytes.len() as u64,
        sha256: hex::encode(Sha256::digest(&tensor.bytes)),
        shape: tensor.shape,
        data_type: tensor.data_type,
    };
    tensor::stage_tensor(&directory.join(name), &tensor.bytes).await?;
    Ok(reference)
}

fn read_output(
    path: &std::path::Path,
    reference: TensorFileReference,
) -> Result<ImageTensor, String> {
    let expected = byte_length(reference.shape, reference.data_type)?;
    if expected != reference.byte_length
        || expected > 256 * 1024 * 1024
        || !tensor::valid_sha256(&reference.sha256)
    {
        return Err("invalid_output: image tensor exceeds the admitted output budget".into());
    }
    let mut file = super::super::artifact::open_regular(path)?;
    if file.metadata().map_err(|error| error.to_string())?.len() != expected {
        return Err("invalid_output: image tensor length differs".into());
    }
    let mut bytes = Vec::with_capacity(expected as usize);
    file.by_ref()
        .take(expected + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 != expected || hex::encode(Sha256::digest(&bytes)) != reference.sha256 {
        return Err("invalid_output: image tensor digest differs".into());
    }
    if reference.data_type == TensorDataType::Float32LittleEndian
        && bytes.chunks_exact(4).any(|chunk| {
            !f32::from_le_bytes(chunk.try_into().expect("Four-byte float")).is_finite()
        })
    {
        return Err("invalid_output: image tensor contains non-finite values".into());
    }
    Ok(ImageTensor {
        bytes,
        shape: reference.shape,
        data_type: reference.data_type,
    })
}

fn byte_length(shape: [u32; 4], dtype: TensorDataType) -> Result<u64, String> {
    if shape.iter().any(|value| *value == 0 || *value > 16384) {
        return Err("invalid_input: invalid tensor dimensions".into());
    }
    let elements = shape
        .iter()
        .try_fold(1_u64, |size, value| size.checked_mul(u64::from(*value)))
        .ok_or("invalid_input: tensor dimensions overflow")?;
    Ok(elements * if dtype == TensorDataType::Uint8 { 1 } else { 4 })
}

fn output_shape(
    operation: &ImageTensorOperation,
    source: [u32; 4],
) -> Result<([u32; 4], TensorDataType), String> {
    let [batch, channels, height, width] = source;
    if batch != 1 || channels != 3 || u64::from(height) * u64::from(width) > 16_777_216 {
        return Err("invalid_input: RGB source exceeds the admitted shape".into());
    }
    Ok(match operation {
        ImageTensorOperation::Upscale if u64::from(height) * u64::from(width) <= 4_194_304 => {
            ([1, 3, height * 4, width * 4], TensorDataType::Uint8)
        }
        ImageTensorOperation::Upscale => {
            return Err("invalid_input: upscale source exceeds the tile budget".into())
        }
        ImageTensorOperation::Inpaint(_) => (source, TensorDataType::Uint8),
        ImageTensorOperation::Select { .. } => ([1, 1, height, width], TensorDataType::Uint8),
        ImageTensorOperation::Depth => ([1, 1, height, width], TensorDataType::Float32LittleEndian),
    })
}

#[cfg(test)]
mod tests;
