use super::{
    super::{admission::Priority, lease::RequestIdentity},
    compute::{stage, ImageTensor},
    session,
};
use crate::{
    ai::contracts::{
        image_compute::{ImageTextRecognition, ImageTextRecognitionParams, RecognizedTextLine},
        runtime::{RuntimeCapability, RuntimeModel},
    },
    local_resources::{
        models::catalog::{ImageModelCatalog, ImageModelVariant},
        preparation::PreparedModels,
    },
};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

/// The worker cannot interrupt one source once it has started; this bounds a source that never ends.
const BUDGET_MS: u64 = 5 * 60 * 1_000;
const MODEL: ImageModelVariant = ImageModelVariant::PpOcrV6Small;

pub(crate) fn spec_id() -> String {
    MODEL.spec_id()
}

pub(crate) async fn recognize<R: Runtime>(
    app: &AppHandle<R>,
    owner_id: &str,
    input_revision: &str,
    input: ImageTensor,
    resources: &PreparedModels,
    cancellation: &CancellationToken,
) -> Result<Vec<RecognizedTextLine>, String> {
    let directory = resources.directory(MODEL.models())?;
    let spec_id = spec_id();
    let mut lease = session::acquire(
        app,
        Priority::Background,
        RuntimeCapability::ImageTextRecognition,
        &spec_id,
        RuntimeModel::ImageVariant {
            path: directory.to_string_lossy().into_owned(),
            variant: MODEL,
        },
        cancellation,
    )
    .await?;
    let request_id = uuid::Uuid::new_v4().to_string();
    let work = lease.work_directory()?.to_owned();
    let source_name = format!("source-{request_id}.tensor");
    let tensor_ref = stage(&work, &source_name, input).await?;
    let params = ImageTextRecognitionParams {
        request_id: request_id.clone(),
        owner_kind: "enrichmentJob".into(),
        owner_id: owner_id.into(),
        attempt_id: owner_id.into(),
        attempt_epoch: 1,
        input_revision: input_revision.into(),
        model_spec_id: spec_id.clone(),
        budget_ms: BUDGET_MS,
        tensor_ref,
    };
    let response = lease
        .request::<ImageTextRecognition>(
            &RequestIdentity {
                request_id: &request_id,
                attempt_epoch: 1,
                budget_ms: BUDGET_MS,
            },
            &params,
            cancellation,
        )
        .await;
    let _ = tokio::fs::remove_file(work.join(source_name)).await;
    let response = response?;
    if response.request_id != request_id
        || response.attempt_epoch != 1
        || response.model_spec_id != spec_id
    {
        lease.discard().await;
        return Err("invalid_output: text recognition response identity differs".into());
    }
    Ok(response.lines)
}
