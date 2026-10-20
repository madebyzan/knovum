use std::{path::Path, time::Duration};

use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::{
    lease::{RequestIdentity, WorkerLease},
    process::WorkerSpecification,
};
use crate::{
    ai::contracts::{
        embedding_wire::{
            EmbeddingEncodeParams, EmbeddingEncoding, EmbeddingExecutionIdentity, EmbeddingRole,
        },
        runtime::{
            RuntimeCapability, RuntimeModel, RuntimeShutdownAcknowledgement, RuntimeShutdownParams,
        },
    },
    local_resources::{
        models::catalog::{self, ModelKind},
        preparation::PreparedModels,
    },
};

pub(crate) async fn encode_embedding<R: Runtime>(
    app: &AppHandle<R>,
    texts: Vec<String>,
    role: EmbeddingRole,
    identity: &EmbeddingExecutionIdentity,
    resources: &PreparedModels,
    cancellation: &CancellationToken,
) -> Result<EmbeddingEncoding, String> {
    let priority = match role {
        EmbeddingRole::Query => super::admission::Priority::Interactive,
        EmbeddingRole::Document => super::admission::Priority::Background,
    };
    let model_directory = embedding_directory(resources)?;
    let mut lease = WorkerLease::acquire(
        app,
        priority,
        WorkerSpecification {
            capability: RuntimeCapability::Embedding,
            capability_spec_id: crate::ai::capabilities::embedding::EMBEDDING_SPEC_ID.into(),
            device_profile: super::device_profiles::embedding::profile()?,
            model: RuntimeModel::Snapshot {
                path: model_directory.to_string_lossy().into_owned(),
            },
        },
        cancellation,
    )
    .await?;
    let request_id = uuid::Uuid::new_v4().to_string();
    let params = EmbeddingEncodeParams {
        request_id: request_id.clone(),
        owner_kind: identity.owner_kind.clone(),
        owner_id: identity.owner_id.clone(),
        attempt_id: identity.attempt_id.clone(),
        attempt_epoch: identity.attempt_epoch,
        input_revision: identity.input_revision.clone(),
        model_spec_id: crate::ai::capabilities::embedding::EMBEDDING_SPEC_ID.into(),
        budget_ms: identity.budget_ms,
        role,
        texts,
        embedding_spec_id: crate::ai::capabilities::embedding::EMBEDDING_SPEC_ID.into(),
    };
    let output: EmbeddingEncoding = lease
        .request(
            &RequestIdentity {
                request_id: &request_id,
                attempt_epoch: identity.attempt_epoch,
                budget_ms: identity.budget_ms,
            },
            &params,
            cancellation,
        )
        .await?;
    if output.request_id != request_id
        || output.attempt_epoch != identity.attempt_epoch
        || output.embedding_spec_id != params.embedding_spec_id
        || output.dimensions != 1024
    {
        lease.discard().await;
        return Err("invalid_output: embedding response identity differs".into());
    }
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
