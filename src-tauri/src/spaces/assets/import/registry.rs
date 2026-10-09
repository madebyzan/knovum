use std::{path::PathBuf, sync::OnceLock};

use tokio_util::sync::CancellationToken;

use crate::{
    file_inspection::ResourceClassification,
    job_registry::{JobAdmissionReservation, JobRegistry},
};

#[derive(Clone)]
pub struct PendingImport {
    pub source: PathBuf,
    pub target: PathBuf,
    pub name: String,
    pub classification: ResourceClassification,
    pub total_bytes: Option<u64>,
    pub cancellation: CancellationToken,
}

fn registry() -> &'static JobRegistry<PendingImport> {
    static REGISTRY: OnceLock<JobRegistry<PendingImport>> = OnceLock::new();
    REGISTRY.get_or_init(|| JobRegistry::new("Space import"))
}

pub fn reserve_maintenance() -> Result<JobAdmissionReservation, String> {
    registry().reserve_maintenance()
}

pub fn insert(job_id: String, pending_import: PendingImport) -> Result<(), String> {
    registry().insert(job_id, pending_import)
}

pub fn get(job_id: &str) -> Result<PendingImport, String> {
    registry().get(job_id)
}

pub fn remove(job_id: &str) -> Result<(), String> {
    registry().remove(job_id)?;
    Ok(())
}

pub fn cancel(job_id: &str) -> Result<(), String> {
    if let Some(pending_import) = registry().remove(job_id)? {
        pending_import.cancellation.cancel();
    }
    Ok(())
}
