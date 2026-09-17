use std::{
    collections::HashSet,
    sync::OnceLock,
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use tauri::{ipc::Channel, AppHandle};
use tokio_util::sync::CancellationToken;

use super::{catalog, install, ModelDownloadProgress};
use crate::job_registry::{JobLease, JobRegistry};

use catalog::ModelKind;

#[derive(Clone)]
struct ActiveModelDownload {
    cancellation: CancellationToken,
    models: Vec<ModelKind>,
    started_at: u64,
}

fn downloads() -> &'static JobRegistry<ActiveModelDownload> {
    static DOWNLOADS: OnceLock<JobRegistry<ActiveModelDownload>> = OnceLock::new();
    DOWNLOADS.get_or_init(|| JobRegistry::new("Model download"))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInstallSnapshot {
    job_id: String,
    models: Vec<ModelKind>,
    started_at: u64,
}

#[tauri::command]
pub async fn install_models(
    app: AppHandle,
    models: Vec<ModelKind>,
    job_id: String,
    on_progress: Channel<ModelDownloadProgress>,
) -> Result<(), String> {
    if models.is_empty() {
        return Err("A model download requires at least one model".to_string());
    }
    if job_id.trim().is_empty() {
        return Err("A model download requires a job identifier".to_string());
    }
    if models.iter().copied().collect::<HashSet<_>>().len() != models.len() {
        return Err("A model download cannot contain duplicate model parts".to_string());
    }
    let cancellation = CancellationToken::new();
    let _lease = begin_installation(&job_id, &models, &cancellation)?;
    let total = models
        .iter()
        .map(|model| catalog::descriptor(*model).download_size)
        .sum::<u64>();
    let mut completed = 0_u64;
    let mut result = Ok(());
    for model in models {
        let descriptor = catalog::descriptor(model);
        let mut report = |progress: ModelDownloadProgress| {
            let downloaded = completed.saturating_add(progress.downloaded);
            on_progress
                .send(ModelDownloadProgress {
                    downloaded,
                    total: Some(total),
                    percent: (total > 0)
                        .then(|| ((downloaded.saturating_mul(100) / total).min(100)) as u8),
                })
                .map_err(|error| error.to_string())
        };
        if let Err(error) = install(&app, descriptor, &mut report, &cancellation).await {
            result = Err(error);
            break;
        }
        completed = completed.saturating_add(descriptor.download_size);
    }
    result
}

#[tauri::command]
pub fn cancel_model_install(job_id: String) -> Result<(), String> {
    let active = downloads()
        .get(&job_id)
        .map_err(|_| format!("Model download job is not active: {job_id}"))?;
    active.cancellation.cancel();
    Ok(())
}

#[tauri::command]
pub fn list_model_installs() -> Result<Vec<ModelInstallSnapshot>, String> {
    Ok(downloads()
        .entries()?
        .into_iter()
        .map(|(job_id, active)| ModelInstallSnapshot {
            job_id,
            models: active.models,
            started_at: active.started_at,
        })
        .collect())
}

fn begin_installation(
    job_id: &str,
    models: &[ModelKind],
    cancellation: &CancellationToken,
) -> Result<JobLease<ActiveModelDownload>, String> {
    let active = ActiveModelDownload {
        cancellation: cancellation.clone(),
        models: models.to_vec(),
        started_at: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_millis() as u64,
    };
    downloads().acquire(job_id, active)
}
