use crate::local_resources::runtimes::python;
use std::{
    path::Path,
    time::{Duration, Instant},
};

use tauri::{AppHandle, Manager, Runtime};
use tokio_util::sync::CancellationToken;

use super::process::Worker;
use crate::{
    ai::contracts::{
        embedding_wire::{
            EmbeddingEncodeParams, EmbeddingEncoding, EmbeddingExecutionIdentity, EmbeddingRole,
        },
        runtime::{
            RuntimeCancelParams, RuntimeCapability, RuntimeInitializeParams, RuntimeModelPaths,
            RuntimeShutdownAcknowledgement, RuntimeShutdownParams, PROTOCOL_VERSION,
        },
    },
    local_resources::{
        models::catalog::{self, ModelKind},
        preparation::PreparedModels,
    },
};

const CANCEL_GRACE: Duration = Duration::from_secs(5);

pub(crate) async fn encode_embedding<R: Runtime>(
    app: &AppHandle<R>,
    texts: Vec<String>,
    role: EmbeddingRole,
    identity: &EmbeddingExecutionIdentity,
    resources: &PreparedModels,
    cancellation: &CancellationToken,
) -> Result<EmbeddingEncoding, String> {
    super::deadline_duration(identity.deadline_ms)?;
    let exit = super::exit_token();
    let priority = match role {
        EmbeddingRole::Query => super::admission::Priority::Interactive,
        EmbeddingRole::Document => super::admission::Priority::Background,
    };
    let runtime =
        python::resolve(app, python::RuntimeProfile::TorchInference)?.ok_or("runtime_missing")?;
    let model_directory = embedding_directory(resources)?;
    let device_profile = super::device_profiles::embedding::profile(
        &runtime,
        cancellation,
        exit,
        identity.deadline_ms,
    )
    .await?;
    super::prepare_worker_for_admission(priority, |worker| {
        worker.matches(
            &runtime.digest,
            model_directory,
            RuntimeCapability::Embedding,
            crate::ai::capabilities::embedding::EMBEDDING_SPEC_ID,
            &device_profile,
        )
    })
    .await?;
    let _permit = super::admission::acquire(
        priority,
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
            model_directory,
            RuntimeCapability::Embedding,
            crate::ai::capabilities::embedding::EMBEDDING_SPEC_ID,
            &device_profile,
        ) {
            worker.terminate().await;
            *guard = None;
        }
    }
    if guard.is_none() {
        let session_id = uuid::Uuid::new_v4().to_string();
        let work_directory = app
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
            capability: RuntimeCapability::Embedding,
            capability_spec_id: crate::ai::capabilities::embedding::EMBEDDING_SPEC_ID.into(),
            device_profile: device_profile.clone(),
            work_directory: work_directory.to_string_lossy().into_owned(),
            model_paths: RuntimeModelPaths {
                embedding: Some(model_directory.to_string_lossy().into_owned()),
                speech_transcription: None,
                speech_alignment: None,
                speech_diarization: None,
                speech_synthesis: None,
                speech_separation: None,
                image_segmentation: None,
                image_processing: None,
            },
        };
        *guard = Some(
            Worker::start(
                &runtime,
                &work_directory,
                model_directory,
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
    let params = EmbeddingEncodeParams {
        request_id: request_id.clone(),
        owner_kind: identity.owner_kind.clone(),
        owner_id: identity.owner_id.clone(),
        attempt_id: identity.attempt_id.clone(),
        attempt_epoch: identity.attempt_epoch,
        input_revision: identity.input_revision.clone(),
        model_spec_id: crate::ai::capabilities::embedding::EMBEDDING_SPEC_ID.into(),
        deadline_ms: identity.deadline_ms,
        role,
        texts,
        embedding_spec_id: crate::ai::capabilities::embedding::EMBEDDING_SPEC_ID.into(),
    };
    let deadline = match worker
        .send_request(
            &request_id,
            "embedding.encode",
            &params,
            identity.deadline_ms,
            cancellation,
            exit,
        )
        .await
    {
        Ok(deadline) => deadline,
        Err(error) => {
            *guard = None;
            return Err(error);
        }
    };
    let encoding = tokio::select! {
        biased;
        _ = exit.cancelled() => {
            cancel(worker, &request_id, identity.attempt_epoch).await;
            *guard = None;
            return Err("cancelled".into());
        }
        _ = cancellation.cancelled() => {
            cancel(worker, &request_id, identity.attempt_epoch).await;
            *guard = None;
            return Err("cancelled".into());
        }
        encoding = tokio::time::timeout_at(deadline, worker.protocol.read_response::<EmbeddingEncoding>(&request_id)) => {
            match encoding {
                Ok(value) => value,
                Err(_) => {
                    cancel(worker, &request_id, identity.attempt_epoch).await;
                    *guard = None;
                    return Err("deadline_exceeded".into());
                }
            }
        }
    };
    worker.last_used = Instant::now();
    let output = match encoding {
        Ok(output) => output,
        Err(error) => {
            worker.terminate().await;
            *guard = None;
            return Err(error);
        }
    };
    if output.request_id != request_id
        || output.attempt_epoch != identity.attempt_epoch
        || output.embedding_spec_id != params.embedding_spec_id
        || output.dimensions != 1024
    {
        worker.terminate().await;
        *guard = None;
        return Err("invalid_output: embedding response identity differs".into());
    }
    drop(guard);
    super::reclaim_when_idle(slot);
    Ok(output)
}

pub(crate) async fn shutdown() {
    let Some(slot) = super::WORKER.get() else {
        return;
    };
    let mut guard = match tokio::time::timeout(Duration::from_secs(7), slot.lock()).await {
        Ok(guard) => guard,
        Err(_) => {
            super::process::force_terminate_active();
            let Ok(guard) = tokio::time::timeout(Duration::from_secs(2), slot.lock()).await else {
                log::warn!(
                    "local inference worker did not release the slot after process termination"
                );
                return;
            };
            guard
        }
    };
    if let Some(worker) = guard.as_mut() {
        let _ = tokio::time::timeout(
            Duration::from_secs(2),
            worker
                .protocol
                .request::<_, RuntimeShutdownAcknowledgement>(
                    &worker.session_id.clone(),
                    "runtime.shutdown",
                    &RuntimeShutdownParams {
                        session_id: worker.session_id.clone(),
                    },
                ),
        )
        .await;
        worker.terminate().await;
    }
    *guard = None;
}

fn embedding_directory(resources: &PreparedModels) -> Result<&Path, String> {
    let config = resources.path(ModelKind::TextEmbeddingConfig)?;
    let directory = config
        .parent()
        .ok_or("model_missing: embedding directory unavailable")?;
    for kind in catalog::TEXT_EMBEDDING_MODELS {
        let expected = directory.join(catalog::descriptor(*kind).file);
        if resources.path(*kind)? != expected {
            return Err("model_corrupt: embedding snapshot files are not co-located".into());
        }
    }
    Ok(directory)
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
            .read_response::<EmbeddingEncoding>(request_id)
            .await
    })
    .await;
    worker.terminate().await;
}
