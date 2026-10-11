mod desktop_tasks;
pub(crate) use desktop_tasks::{snapshot as desktop_tasks, subscribe};

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex, OnceLock},
    time::{SystemTime, UNIX_EPOCH},
};

use tokio_util::sync::CancellationToken;

use super::{
    contracts::{
        ResourcePreparationProgress, ResourcePreparationSnapshot, ResourcePreparationStage,
        ResourcePreparationState, ResourcePreparationTarget,
    },
    RESOURCE_PREPARATION_CANCELLED,
};
use crate::foundation::app_error::AppError;

const COMPLETED_LIMIT: usize = 32;
const COMPLETED_TTL_MS: u64 = 10 * 60 * 1_000;

#[derive(Clone)]
struct ActivePreparation {
    cancellation: CancellationToken,
    snapshot: Arc<Mutex<ResourcePreparationSnapshot>>,
}

pub(crate) struct PreparationRequest {
    job_id: String,
    active: ActivePreparation,
    finished: bool,
}

pub(crate) struct PreparationAdmissionReservation {
    registry: &'static Mutex<RegistryState>,
}

impl Drop for PreparationAdmissionReservation {
    fn drop(&mut self) {
        match self.registry.lock() {
            Ok(mut registry) => registry.maintenance_reserved = false,
            Err(_) => log::error!("Failed to release resource preparation maintenance admission"),
        }
    }
}

impl PreparationRequest {
    pub(crate) fn cancellation(&self) -> &CancellationToken {
        &self.active.cancellation
    }

    pub(crate) fn update(&self, progress: ResourcePreparationProgress) {
        match self.active.snapshot.lock() {
            Ok(mut snapshot) => snapshot.progress = progress,
            Err(_) => log::error!("Resource preparation snapshot lock poisoned"),
        }
        desktop_tasks::announce();
    }

    pub(crate) fn finish(mut self, error: Option<&String>) -> Result<(), String> {
        let completion = finish_request(&self.job_id, &self.active, error);
        self.finished = true;
        completion
    }
}

impl Drop for PreparationRequest {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        self.active.cancellation.cancel();
        let error = RESOURCE_PREPARATION_CANCELLED.to_string();
        if let Err(failure) = finish_request(&self.job_id, &self.active, Some(&error)) {
            log::error!("Unable to finish resource preparation: {failure}");
        }
    }
}

pub(crate) fn begin(
    job_id: &str,
    title: &str,
    target: ResourcePreparationTarget,
) -> Result<PreparationRequest, String> {
    if job_id.trim().is_empty() {
        return Err("Resource preparation requires a job identifier".into());
    }
    if title.trim().is_empty() {
        return Err("Resource preparation requires a task title".into());
    }
    let active = ActivePreparation {
        cancellation: CancellationToken::new(),
        snapshot: Arc::new(Mutex::new(ResourcePreparationSnapshot {
            job_id: job_id.to_owned(),
            title: title.to_owned(),
            progress: ResourcePreparationProgress {
                stage: ResourcePreparationStage::Queued,
                downloaded: 0,
                total: None,
                percent: None,
            },
            target,
            started_at: now()?,
            completed_at: None,
            state: ResourcePreparationState::Queued,
            error: None,
        })),
    };
    let mut registry = registry()
        .lock()
        .map_err(|_| "Resource preparation registry lock poisoned".to_string())?;
    if registry.maintenance_reserved {
        return Err("Resource preparation is closed for maintenance".into());
    }
    if registry.active.contains_key(job_id) {
        return Err(format!("Resource preparation job already exists: {job_id}"));
    }
    active
        .snapshot
        .lock()
        .map_err(|_| "Resource preparation snapshot lock poisoned".to_string())?
        .state = ResourcePreparationState::Preparing;
    registry.completed.retain(|item| item.job_id != job_id);
    registry.active.insert(job_id.to_owned(), active.clone());
    desktop_tasks::announce();
    Ok(PreparationRequest {
        job_id: job_id.to_owned(),
        active,
        finished: false,
    })
}

pub(crate) fn cancel(job_id: &str) -> Result<(), String> {
    let registry = registry()
        .lock()
        .map_err(|_| "Resource preparation registry lock poisoned".to_string())?;
    if let Some(active) = registry.active.get(job_id) {
        active.cancellation.cancel();
    }
    Ok(())
}

pub(crate) fn reserve_maintenance() -> Result<PreparationAdmissionReservation, String> {
    let state = registry();
    let mut registry = state
        .lock()
        .map_err(|_| "Resource preparation registry lock poisoned".to_string())?;
    if registry.maintenance_reserved {
        return Err("Resource preparation maintenance admission is already reserved".into());
    }
    if !registry.active.is_empty() {
        return Err(
            "Wait for active model preparation tasks to finish before data maintenance".into(),
        );
    }
    registry.maintenance_reserved = true;
    Ok(PreparationAdmissionReservation { registry: state })
}

pub(crate) fn list() -> Result<Vec<ResourcePreparationSnapshot>, String> {
    let cutoff = now()?.saturating_sub(COMPLETED_TTL_MS);
    let mut registry = registry()
        .lock()
        .map_err(|_| "Resource preparation registry lock poisoned".to_string())?;
    registry
        .completed
        .retain(|item| item.completed_at.is_none_or(|time| time >= cutoff));
    let active = registry
        .active
        .values()
        .map(|item| {
            item.snapshot
                .lock()
                .map(|value| value.clone())
                .map_err(|_| "Resource preparation snapshot lock poisoned".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(active
        .into_iter()
        .chain(registry.completed.iter().cloned())
        .collect())
}

fn finish_request(
    job_id: &str,
    active: &ActivePreparation,
    error: Option<&String>,
) -> Result<(), String> {
    let mut registry = registry()
        .lock()
        .map_err(|_| "Resource preparation registry lock poisoned".to_string())?;
    let mut snapshot = active
        .snapshot
        .lock()
        .map_err(|_| "Resource preparation snapshot lock poisoned".to_string())?;
    let completed_at = now();
    snapshot.completed_at = completed_at.as_ref().ok().copied();
    let failure = error
        .cloned()
        .or_else(|| completed_at.as_ref().err().cloned());
    match failure.as_ref() {
        None => {
            snapshot.state = ResourcePreparationState::Ready;
            snapshot.error = None;
        }
        Some(_) if active.cancellation.is_cancelled() => {
            snapshot.state = ResourcePreparationState::Cancelled;
            snapshot.error = None;
        }
        Some(message) => {
            snapshot.state = ResourcePreparationState::Failed;
            snapshot.error = Some(AppError::from(message.clone()));
        }
    }
    let terminal = snapshot.clone();
    drop(snapshot);
    registry.active.remove(job_id);
    registry.completed.push_front(terminal);
    registry.completed.truncate(COMPLETED_LIMIT);
    desktop_tasks::announce();
    completed_at.map(|_| ())
}

fn now() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis()
        .try_into()
        .map_err(|error: std::num::TryFromIntError| error.to_string())
}

#[derive(Default)]
struct RegistryState {
    maintenance_reserved: bool,
    active: HashMap<String, ActivePreparation>,
    completed: VecDeque<ResourcePreparationSnapshot>,
}

fn registry() -> &'static Mutex<RegistryState> {
    static REGISTRY: OnceLock<Mutex<RegistryState>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(RegistryState::default()))
}

#[cfg(test)]
mod tests;
