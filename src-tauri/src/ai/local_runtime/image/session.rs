use super::super::{admission, process::Worker, WorkerSlot};
use crate::{
    ai::contracts::runtime::{
        RuntimeCancelParams, RuntimeCapability, RuntimeDeviceProfile, RuntimeInitializeParams,
        RuntimeModelPaths, PROTOCOL_VERSION,
    },
    local_resources::runtimes::python,
};
use serde::{de::DeserializeOwned, Serialize};
use std::{path::Path, time::Instant};
use tauri::{AppHandle, Manager, Runtime};
use tokio_util::sync::CancellationToken;

pub(super) struct ImageWorkerLease {
    pub guard: tokio::sync::OwnedMutexGuard<Option<Worker>>,
    _permit: admission::Permit,
    slot: WorkerSlot,
}

impl Drop for ImageWorkerLease {
    fn drop(&mut self) {
        super::super::reclaim_when_idle(self.slot.clone());
    }
}

pub(super) struct ImageWorkerConfiguration<'a> {
    pub model_path: &'a Path,
    pub paths: RuntimeModelPaths,
    pub capability: RuntimeCapability,
    pub spec_id: &'a str,
    pub deadline_ms: u64,
}

pub(super) async fn acquire<R: Runtime>(
    app: &AppHandle<R>,
    configuration: ImageWorkerConfiguration<'_>,
    cancellation: &CancellationToken,
) -> Result<ImageWorkerLease, String> {
    let ImageWorkerConfiguration {
        model_path,
        paths,
        capability,
        spec_id,
        deadline_ms,
    } = configuration;
    let exit = super::super::exit_token();
    let runtime =
        python::resolve(app, python::RuntimeProfile::LightInference)?.ok_or("runtime_missing")?;
    let threads = super::super::compute_budget::ComputeBudget::detect()?.model_threads;
    let profile_name = match capability {
        RuntimeCapability::ImageSegmentation => "cpu-f32",
        RuntimeCapability::ImageUpscaling => "image-upscaling-cpu",
        RuntimeCapability::ImageInpainting => "image-inpainting-cpu",
        RuntimeCapability::ImageSelection => "image-selection-cpu",
        RuntimeCapability::ImageDepth => "image-depth-cpu",
        _ => return Err("runtime_mismatch: not an image capability".into()),
    };
    let profile = RuntimeDeviceProfile {
        device: "cpu".into(),
        execution_profile_id: format!("{profile_name}-threads{threads}-v1"),
        cpu_threads: threads,
    };
    super::super::prepare_worker_for_admission(admission::Priority::Interactive, |worker| {
        worker.matches(&runtime.digest, model_path, capability, spec_id, &profile)
    })
    .await?;
    let permit = admission::acquire(
        admission::Priority::Interactive,
        cancellation,
        exit,
        super::super::deadline_duration(deadline_ms)?,
    )
    .await?;
    let slot = super::super::worker_slot();
    let mut guard = tokio::select! {
        biased;
        _ = exit.cancelled() => return Err("cancelled".into()),
        _ = cancellation.cancelled() => return Err("cancelled".into()),
        guard = slot.clone().lock_owned() => guard,
    };
    if let Some(worker) = guard.as_mut() {
        if !worker.matches(&runtime.digest, model_path, capability, spec_id, &profile) {
            worker.terminate().await;
            *guard = None;
        }
    }
    if guard.is_none() {
        let session_id = uuid::Uuid::new_v4().to_string();
        let work = app
            .path()
            .app_cache_dir()
            .map_err(|error| error.to_string())?
            .join("inference")
            .join(&session_id);
        let params = RuntimeInitializeParams {
            runtime_manifest_path: runtime.manifest_path.to_string_lossy().into_owned(),
            protocol_version: PROTOCOL_VERSION,
            runtime_digest: runtime.digest.clone(),
            session_id,
            capability,
            capability_spec_id: spec_id.into(),
            device_profile: profile,
            work_directory: work.to_string_lossy().into_owned(),
            model_paths: paths,
        };
        *guard = Some(
            Worker::start(
                &runtime,
                &work,
                model_path,
                params,
                deadline_ms,
                cancellation,
                exit,
            )
            .await?,
        );
    }
    Ok(ImageWorkerLease {
        guard,
        _permit: permit,
        slot,
    })
}

pub(super) struct ImageRequestIdentity<'a> {
    pub request_id: &'a str,
    pub attempt_epoch: u64,
    pub deadline_ms: u64,
}

pub(super) async fn request<P: Serialize, O: DeserializeOwned>(
    worker: &mut Worker,
    identity: ImageRequestIdentity<'_>,
    params: &P,
    cancellation: &CancellationToken,
) -> Result<O, String> {
    let exit = super::super::exit_token();
    let deadline = worker
        .send_request(
            identity.request_id,
            worker.capability.method(),
            params,
            identity.deadline_ms,
            cancellation,
            exit,
        )
        .await?;
    let (response, interrupted) = tokio::select! {
        biased;
        _ = exit.cancelled() => (Err("cancelled".into()), true),
        _ = cancellation.cancelled() => (Err("cancelled".into()), true),
        response = tokio::time::timeout_at(deadline, worker.protocol.read_response::<O>(identity.request_id)) => match response {
            Ok(response) => (response, false),
            Err(_) => (Err("deadline_exceeded".into()), true),
        },
    };
    if response.is_err() {
        if interrupted {
            let _ = tokio::time::timeout(std::time::Duration::from_secs(5), async {
                worker
                    .protocol
                    .notify(
                        "runtime.cancel",
                        &RuntimeCancelParams {
                            target_request_id: identity.request_id.into(),
                            attempt_epoch: identity.attempt_epoch,
                        },
                    )
                    .await?;
                worker
                    .protocol
                    .read_response::<O>(identity.request_id)
                    .await
            })
            .await;
        }
        worker.terminate().await;
    } else {
        worker.last_used = Instant::now();
    }
    response
}
