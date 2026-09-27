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
use crate::command_error::CommandError;

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

impl PreparationRequest {
    pub(crate) fn cancellation(&self) -> &CancellationToken {
        &self.active.cancellation
    }

    pub(crate) fn update(&self, progress: ResourcePreparationProgress) {
        match self.active.snapshot.lock() {
            Ok(mut snapshot) => progress_snapshot(&mut snapshot, progress),
            Err(_) => log::error!("Resource preparation snapshot lock poisoned"),
        }
    }

    pub(crate) fn finish(mut self, error: Option<&String>) {
        finish_request(&self.job_id, &self.active, error);
        self.finished = true;
    }
}

impl Drop for PreparationRequest {
    fn drop(&mut self) {
        if self.finished {
            return;
        }
        self.active.cancellation.cancel();
        let error = RESOURCE_PREPARATION_CANCELLED.to_string();
        finish_request(&self.job_id, &self.active, Some(&error));
    }
}

pub(crate) fn begin(
    job_id: &str,
    title: &str,
    target: ResourcePreparationTarget,
) -> Result<PreparationRequest, String> {
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
    if registry.active.contains_key(job_id) {
        return Err(format!("Resource preparation job already exists: {job_id}"));
    }
    registry.completed.retain(|item| item.job_id != job_id);
    registry.active.insert(job_id.to_owned(), active.clone());
    if let Ok(mut snapshot) = active.snapshot.lock() {
        snapshot.state = ResourcePreparationState::Preparing;
    }
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

pub(crate) fn list() -> Result<Vec<ResourcePreparationSnapshot>, String> {
    let cutoff = now()?.saturating_sub(COMPLETED_TTL_MS);
    let mut registry = registry()
        .lock()
        .map_err(|_| "Resource preparation registry lock poisoned".to_string())?;
    registry
        .completed
        .retain(|item| item.completed_at.is_some_and(|time| time >= cutoff));
    let active = registry
        .active
        .values()
        .filter_map(|item| item.snapshot.lock().ok().map(|value| value.clone()))
        .collect::<Vec<_>>();
    Ok(active
        .into_iter()
        .chain(registry.completed.iter().cloned())
        .collect())
}

fn progress_snapshot(
    snapshot: &mut ResourcePreparationSnapshot,
    progress: ResourcePreparationProgress,
) {
    snapshot.progress = progress;
}

fn finish_request(job_id: &str, active: &ActivePreparation, error: Option<&String>) {
    let Ok(mut registry) = registry().lock() else {
        log::error!("Resource preparation registry lock poisoned");
        return;
    };
    let Ok(mut snapshot) = active.snapshot.lock() else {
        log::error!("Resource preparation snapshot lock poisoned");
        registry.active.remove(job_id);
        return;
    };
    snapshot.completed_at = now().ok();
    match error {
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
            snapshot.error = Some(CommandError::from(message.clone()));
        }
    }
    let terminal = snapshot.clone();
    drop(snapshot);
    registry.active.remove(job_id);
    registry.completed.push_front(terminal);
    registry.completed.truncate(COMPLETED_LIMIT);
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
    active: HashMap<String, ActivePreparation>,
    completed: VecDeque<ResourcePreparationSnapshot>,
}

fn registry() -> &'static Mutex<RegistryState> {
    static REGISTRY: OnceLock<Mutex<RegistryState>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(RegistryState::default()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepted_request_can_be_cancelled_before_runner_starts() {
        let job_id = format!("resource-acceptance:{}", uuid::Uuid::new_v4());
        let request = begin(
            &job_id,
            "Preparing media import",
            ResourcePreparationTarget::MediaImportTools,
        )
        .unwrap();

        cancel(&job_id).unwrap();

        assert!(request.cancellation().is_cancelled());
        let error = RESOURCE_PREPARATION_CANCELLED.to_string();
        request.finish(Some(&error));
        let snapshot = list()
            .unwrap()
            .into_iter()
            .find(|snapshot| snapshot.job_id == job_id)
            .unwrap();
        assert!(matches!(
            snapshot.state,
            ResourcePreparationState::Cancelled
        ));
    }
}
