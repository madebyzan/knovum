//! Warm resident process pool for Knovum's built-in Python Agent runtime.
//!
//! Cold-booting Python and importing LangGraph/DeepAgents takes several seconds per turn.
//! Keeping one initialized runtime process alive across clean turns eliminates startup overhead
//! while preserving strict per-turn fault isolation: any error, cancellation, pause, or timeout
//! discards and terminates the process instead of returning it to the pool, and idle processes
//! are reclaimed automatically after [`IDLE_TIMEOUT`] or on application exit.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError, RwLock},
    time::Duration,
};

use futures::future::BoxFuture;
use serde_json::Value;
use tauri::Manager;
use tokio::sync::Notify;

use super::{Host, Peer, Process};
use crate::foundation::subprocess::process_tree::KillHandle;

const IDLE_TIMEOUT: Duration = Duration::from_secs(15 * 60);

/// Returns the canonical SQLite checkpoint file shared by prepared and leased built-in Agent
/// runtime processes.
pub(crate) fn checkpoint_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?
        .join("agent-runtime/checkpoints.sqlite"))
}

/// Thread-safe swappable host bridge bound to one warm Agent runtime process.
/// Locks are never held across `.await` boundaries.
pub(crate) struct HostSlot {
    active: RwLock<Option<Arc<dyn Host>>>,
}

impl HostSlot {
    pub(crate) fn new() -> Self {
        Self {
            active: RwLock::new(None),
        }
    }

    pub(crate) fn bind(&self, host: Arc<dyn Host>) {
        *self.active.write().unwrap_or_else(PoisonError::into_inner) = Some(host);
    }

    pub(crate) fn unbind(&self) {
        *self.active.write().unwrap_or_else(PoisonError::into_inner) = None;
    }

    fn current(&self) -> Result<Arc<dyn Host>, String> {
        self.active
            .read()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
            .ok_or_else(|| "agent-runtime-host-unbound".to_owned())
    }
}

impl Host for HostSlot {
    fn dispatch(&self, method: String, params: Value) -> BoxFuture<'_, Result<Value, String>> {
        let host = self.current();
        Box::pin(async move { host?.dispatch(method, params).await })
    }

    fn notify(&self, method: &str, params: Value) -> Result<(), String> {
        self.current()?.notify(method, params)
    }
}

struct ResidentProcess {
    checkpoint_path: PathBuf,
    slot: Arc<HostSlot>,
    process: Process,
}

#[derive(Default)]
struct PoolState {
    idle: Option<ResidentProcess>,
    generation: u64,
    next_lease_id: u64,
    active_leases: HashMap<u64, KillHandle>,
    preparing: Option<Arc<Notify>>,
    shutting_down: bool,
}

static POOL: OnceLock<Mutex<PoolState>> = OnceLock::new();

fn pool_state() -> MutexGuard<'static, PoolState> {
    POOL.get_or_init(Default::default)
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

/// Parks `resident` in the idle slot unless the pool is shutting down, returning any displaced
/// (or rejected-during-shutdown) process for immediate termination outside the lock.
fn park_idle(resident: ResidentProcess) -> Option<ResidentProcess> {
    let (to_terminate, generation) = {
        let mut state = pool_state();
        if state.shutting_down {
            return Some(resident);
        }
        state.generation = state.generation.wrapping_add(1);
        let generation = state.generation;
        let replaced = state.idle.replace(resident);
        (replaced, generation)
    };
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(IDLE_TIMEOUT).await;
        let evicted = {
            let mut state = pool_state();
            if state.generation == generation {
                state.idle.take()
            } else {
                None
            }
        };
        if let Some(mut resident) = evicted {
            resident.process.terminate().await;
        }
    });
    to_terminate
}

/// Exclusive lease on a warm (or freshly started) built-in Agent process.
/// Dropping a lease without calling [`Self::release_healthy`] unbinds its host and terminates the
/// underlying process tree, guaranteeing unhealthy processes are never recycled.
pub(crate) struct LeasedProcess {
    lease_id: Option<u64>,
    resident: Option<ResidentProcess>,
}

impl LeasedProcess {
    fn new(resident: ResidentProcess) -> Self {
        let handle = resident.process.kill_handle();
        let lease_id = handle.map(|handle| {
            let mut state = pool_state();
            if state.shutting_down {
                handle.terminate();
                return 0;
            }
            state.next_lease_id = state.next_lease_id.wrapping_add(1).max(1);
            let id = state.next_lease_id;
            state.active_leases.insert(id, handle);
            id
        });
        Self {
            lease_id: lease_id.filter(|id| *id != 0),
            resident: Some(resident),
        }
    }

    fn unregister_lease(&mut self) {
        if let Some(id) = self.lease_id.take() {
            pool_state().active_leases.remove(&id);
        }
    }

    pub(crate) fn peer(&self) -> &Peer {
        &self
            .resident
            .as_ref()
            .expect("active lease owns its resident process")
            .process
            .peer
    }

    /// Returns a healthy, cleanly completed process to the idle pool for reuse on the next turn.
    pub(crate) async fn release_healthy(mut self) {
        self.unregister_lease();
        let Some(mut resident) = self.resident.take() else {
            return;
        };
        resident.slot.unbind();
        if !resident.process.is_alive() {
            resident.process.terminate().await;
            return;
        }
        if let Some(mut displaced) = park_idle(resident) {
            displaced.process.terminate().await;
        }
    }

    /// Unbinds the host and terminates the process tree immediately.
    pub(crate) async fn discard(mut self) {
        self.unregister_lease();
        if let Some(mut resident) = self.resident.take() {
            resident.slot.unbind();
            resident.process.terminate().await;
        }
    }
}

impl Drop for LeasedProcess {
    fn drop(&mut self) {
        self.unregister_lease();
        if let Some(resident) = self.resident.take() {
            resident.slot.unbind();
            drop(resident.process);
        }
    }
}

fn take_idle_or_preparation() -> (
    Option<ResidentProcess>,
    Option<tokio::sync::futures::OwnedNotified>,
) {
    let mut state = pool_state();
    if let Some(resident) = state.idle.take() {
        state.generation = state.generation.wrapping_add(1);
        return (Some(resident), None);
    }
    let waiting = state
        .preparing
        .as_ref()
        .map(|notify| Arc::clone(notify).notified_owned());
    (None, waiting)
}

/// Acquires a warm initialized Agent runtime process for `checkpoint_path` or starts a new one,
/// binding `host` for the duration of the turn. If a background [`prepare`] is already starting
/// a process, joins it instead of launching a duplicate Python process.
pub(crate) async fn acquire(
    app: &tauri::AppHandle,
    checkpoint_path: &Path,
    host: Arc<dyn Host>,
) -> Result<LeasedProcess, String> {
    let (mut cached, waiting) = take_idle_or_preparation();
    if cached.is_none() {
        if let Some(notified) = waiting {
            notified.await;
            cached = take_idle_or_preparation().0;
        }
    }
    if let Some(mut resident) = cached {
        if resident.checkpoint_path == checkpoint_path && resident.process.is_alive() {
            resident.slot.bind(host);
            return Ok(LeasedProcess::new(resident));
        }
        resident.slot.unbind();
        resident.process.terminate().await;
    }

    let slot = Arc::new(HostSlot::new());
    slot.bind(host);
    let process = Process::start(app, checkpoint_path, slot.clone()).await?;
    Ok(LeasedProcess::new(ResidentProcess {
        checkpoint_path: checkpoint_path.to_path_buf(),
        slot,
        process,
    }))
}

/// Lazily prepares one idle built-in Agent process in the background when none is cached.
/// Safe to call repeatedly (e.g., when opening the Conversation pane with a built-in model).
pub(crate) fn prepare(app: tauri::AppHandle) {
    let Ok(checkpoint_path) = checkpoint_path(&app) else {
        return;
    };
    let notify = {
        let mut state = pool_state();
        if state.shutting_down || state.preparing.is_some() {
            return;
        }
        if let Some(resident) = state.idle.as_mut() {
            if resident.checkpoint_path == checkpoint_path && resident.process.is_alive() {
                return;
            }
        }
        let notify = Arc::new(Notify::new());
        state.preparing = Some(Arc::clone(&notify));
        notify
    };
    tauri::async_runtime::spawn(async move {
        struct PreparationGuard(Arc<Notify>);
        impl Drop for PreparationGuard {
            fn drop(&mut self) {
                let mut state = pool_state();
                if state
                    .preparing
                    .as_ref()
                    .is_some_and(|current| Arc::ptr_eq(current, &self.0))
                {
                    state.preparing = None;
                }
                drop(state);
                self.0.notify_waiters();
            }
        }
        let _guard = PreparationGuard(notify);
        let slot = Arc::new(HostSlot::new());
        let mut process = match Process::start(&app, &checkpoint_path, slot.clone()).await {
            Ok(process) => process,
            Err(error) => {
                log::debug!("Built-in Agent background preparation skipped: {error}");
                return;
            }
        };
        if !process.is_alive() {
            process.terminate().await;
            return;
        }
        let resident = ResidentProcess {
            checkpoint_path,
            slot,
            process,
        };
        if let Some(mut displaced) = park_idle(resident) {
            displaced.process.terminate().await;
        }
    });
}

/// Marks the pool as shutting down and terminates any idle or actively leased built-in Agent
/// runtime processes when the application exits.
pub(crate) async fn shutdown() {
    let (resident, notify, active_handles) = {
        let mut state = pool_state();
        state.shutting_down = true;
        state.generation = state.generation.wrapping_add(1);
        let handles: Vec<KillHandle> = state
            .active_leases
            .drain()
            .map(|(_, handle)| handle)
            .collect();
        (state.idle.take(), state.preparing.take(), handles)
    };
    if let Some(notify) = notify {
        notify.notify_waiters();
    }
    for handle in active_handles {
        handle.terminate();
    }
    if let Some(mut resident) = resident {
        resident.slot.unbind();
        resident.process.terminate().await;
    }
}
