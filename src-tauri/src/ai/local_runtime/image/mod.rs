use crate::local_resources::runtimes::python;
use tensor::{encode_f32, read_mask, stage_tensor, valid_sha256, validate_input};
mod tensor;
use std::time::{Duration, Instant};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, Runtime};
use tokio_util::sync::CancellationToken;

use super::process::Worker;
use crate::{
    ai::contracts::{
        image_wire::{
            ImageSegmentParams, ImageSegmentation, ImageSegmentationExecutionIdentity,
            TensorDataType, TensorFileReference, IMAGE_SEGMENTATION_INPUT_BYTES,
            IMAGE_SEGMENTATION_MASK_BYTES, IMAGE_SEGMENTATION_RESOLUTION,
            IMAGE_SEGMENTATION_SPEC_ID,
        },
        runtime::{
            RuntimeCancelParams, RuntimeCapability, RuntimeDeviceProfile, RuntimeInitializeParams,
            RuntimeModelPaths, PROTOCOL_VERSION,
        },
    },
    local_resources::{models::catalog::ModelKind, preparation::PreparedModels},
};

const CANCEL_GRACE: Duration = Duration::from_secs(5);
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
    let exit = super::exit_token();
    let runtime =
        python::resolve(app, python::RuntimeProfile::Inference)?.ok_or("runtime_missing")?;
    let model_path = resources.path(ModelKind::ImageSegmentation)?;
    let threads = std::thread::available_parallelism()
        .map(|count| count.get().min(4) as u8)
        .unwrap_or(1);
    let device_profile = RuntimeDeviceProfile {
        device: "cpu".into(),
        execution_profile_id: format!("cpu-f32-threads{threads}-v1"),
        cpu_threads: threads,
    };
    super::prepare_worker_for_admission(super::admission::Priority::Interactive, |worker| {
        worker.matches(
            &runtime.digest,
            model_path,
            RuntimeCapability::ImageSegmentation,
            IMAGE_SEGMENTATION_SPEC_ID,
            &device_profile,
        )
    })
    .await?;
    let _permit = super::admission::acquire(
        super::admission::Priority::Interactive,
        cancellation,
        exit,
        super::deadline_duration(identity.deadline_ms)?,
    )
    .await?;
    let slot = super::worker_slot();
    let mut guard = tokio::select! {
        biased;
        _ = exit.cancelled() => return Err("cancelled".into()),
        _ = cancellation.cancelled() => return Err("cancelled".into()),
        guard = slot.lock() => guard,
    };
    if let Some(worker) = guard.as_mut() {
        if !worker.matches(
            &runtime.digest,
            model_path,
            RuntimeCapability::ImageSegmentation,
            IMAGE_SEGMENTATION_SPEC_ID,
            &device_profile,
        ) {
            worker.terminate().await;
            *guard = None;
        }
    }
    if guard.is_none() {
        let session_id = uuid::Uuid::new_v4().to_string();
        let work_directory = inference_directory(app, &session_id)?;
        let params = RuntimeInitializeParams {
            runtime_manifest_path: runtime.manifest_path.to_string_lossy().into_owned(),
            protocol_version: PROTOCOL_VERSION,
            runtime_digest: runtime.digest.clone(),
            session_id,
            capability: RuntimeCapability::ImageSegmentation,
            capability_spec_id: IMAGE_SEGMENTATION_SPEC_ID.into(),
            device_profile: device_profile.clone(),
            work_directory: work_directory.to_string_lossy().into_owned(),
            model_paths: RuntimeModelPaths {
                embedding: None,
                speech_transcription: None,
                speech_alignment: None,
                speech_diarization: None,
                speech_synthesis: None,
                speech_separation: None,
                image_segmentation: Some(model_path.to_string_lossy().into_owned()),
            },
        };
        *guard = Some(
            Worker::start(
                &runtime,
                &work_directory,
                model_path,
                params,
                identity.deadline_ms,
                cancellation,
                exit,
            )
            .await?,
        );
    }
    let worker = guard.as_mut().ok_or("worker_crashed")?;
    let request_id = uuid::Uuid::new_v4().to_string();
    let work_directory = inference_directory(app, &worker.session_id)?;
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
        model_spec_id: IMAGE_SEGMENTATION_SPEC_ID.into(),
        deadline_ms: identity.deadline_ms,
        tensor_ref: TensorFileReference {
            file_name: tensor_name,
            byte_length: IMAGE_SEGMENTATION_INPUT_BYTES,
            sha256: tensor_sha256,
            shape: INPUT_SHAPE,
            data_type: TensorDataType::Float32LittleEndian,
        },
    };
    let request = request(
        worker,
        &request_id,
        identity.attempt_epoch,
        identity.deadline_ms,
        &params,
        cancellation,
        exit,
    )
    .await;
    let _ = tokio::fs::remove_file(&tensor_path).await;
    let output = match request {
        Ok(output) => output,
        Err(error) => {
            worker.terminate().await;
            *guard = None;
            return Err(error);
        }
    };
    let expected_mask = format!("mask-{request_id}.f32");
    if output.request_id != request_id
        || output.attempt_epoch != identity.attempt_epoch
        || output.model_spec_id != IMAGE_SEGMENTATION_SPEC_ID
        || output.mask_ref.file_name != expected_mask
        || output.mask_ref.byte_length != IMAGE_SEGMENTATION_MASK_BYTES
        || !valid_sha256(&output.mask_ref.sha256)
        || output.mask_ref.shape != MASK_SHAPE
        || !matches!(
            output.mask_ref.data_type,
            TensorDataType::Float32LittleEndian
        )
    {
        worker.terminate().await;
        *guard = None;
        return Err("invalid_output: image segmentation response identity differs".into());
    }
    let mask_path = work_directory.join(expected_mask);
    let mask_sha256 = output.mask_ref.sha256;
    let mask = crate::blocking::run(move || read_mask(&mask_path, &mask_sha256)).await;
    let mask_path = work_directory.join(format!("mask-{request_id}.f32"));
    let _ = tokio::fs::remove_file(&mask_path).await;
    let mask = match mask {
        Ok(mask) => mask,
        Err(error) => {
            worker.terminate().await;
            *guard = None;
            return Err(error);
        }
    };
    worker.last_used = Instant::now();
    drop(guard);
    super::reclaim_when_idle(slot);
    Ok(mask)
}

async fn request(
    worker: &mut Worker,
    request_id: &str,
    attempt_epoch: u64,
    deadline_ms: u64,
    params: &ImageSegmentParams,
    cancellation: &CancellationToken,
    exit: &CancellationToken,
) -> Result<ImageSegmentation, String> {
    let deadline = worker
        .send_request(
            request_id,
            RuntimeCapability::ImageSegmentation.method(),
            params,
            deadline_ms,
            cancellation,
            exit,
        )
        .await?;
    tokio::select! {
        biased;
        _ = exit.cancelled() => {
            cancel(worker, request_id, attempt_epoch).await;
            Err("cancelled".into())
        }
        _ = cancellation.cancelled() => {
            cancel(worker, request_id, attempt_epoch).await;
            Err("cancelled".into())
        }
        result = tokio::time::timeout_at(
            deadline,
            worker.protocol.read_response::<ImageSegmentation>(request_id),
        ) => match result {
            Ok(value) => value,
            Err(_) => {
                cancel(worker, request_id, attempt_epoch).await;
                Err("deadline_exceeded".into())
            }
        }
    }
}

async fn cancel(worker: &mut Worker, request_id: &str, attempt_epoch: u64) {
    let _ = tokio::time::timeout(CANCEL_GRACE, async {
        worker
            .protocol
            .notify(
                "runtime.cancel",
                &RuntimeCancelParams {
                    target_request_id: request_id.into(),
                    attempt_epoch,
                },
            )
            .await?;
        worker
            .protocol
            .read_response::<ImageSegmentation>(request_id)
            .await
    })
    .await;
    worker.terminate().await;
}

fn inference_directory<R: Runtime>(
    app: &AppHandle<R>,
    session_id: &str,
) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_cache_dir()
        .map_err(|error| error.to_string())
        .map(|path| path.join("inference").join(session_id))
}
