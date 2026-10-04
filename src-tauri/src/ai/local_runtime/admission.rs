use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};

use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

use crate::ai::contracts::runtime::{RuntimeCapability, RuntimeDeviceProfile};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Priority {
    Interactive,
    Background,
}

struct Waiter {
    id: u64,
    since: Instant,
    grant: oneshot::Sender<Permit>,
}

#[derive(Default)]
struct State {
    busy: bool,
    next_id: u64,
    reserved_memory_bytes: u64,
    interactive: VecDeque<Waiter>,
    background: VecDeque<Waiter>,
}

struct Admission {
    state: Mutex<State>,
}

pub(crate) struct Permit {
    admission: Arc<Admission>,
    active: bool,
}

pub(super) struct MemoryReservation {
    admission: Arc<Admission>,
    bytes: u64,
    active: bool,
}

static ADMISSION: OnceLock<Arc<Admission>> = OnceLock::new();
const BACKGROUND_MAX_WAIT: Duration = Duration::from_secs(30);
const ACTIVE_RUNTIME_HEADROOM_BYTES: u64 = 1024 * 1024 * 1024;

fn admission() -> Arc<Admission> {
    ADMISSION
        .get_or_init(|| {
            Arc::new(Admission {
                state: Mutex::new(State::default()),
            })
        })
        .clone()
}

pub(super) fn reserve_memory(
    capability: RuntimeCapability,
    device_profile: &RuntimeDeviceProfile,
) -> Result<MemoryReservation, String> {
    let requirement = super::device_profiles::memory_requirement(capability, device_profile)?;
    let peak_bytes = requirement.peak_bytes();
    let snapshot = super::device_profiles::memory_snapshot()?;
    let admission = admission();
    let mut state = admission.state.lock().map_err(|_| "resource_busy")?;
    let requested = validate_memory_reservation(
        peak_bytes,
        requirement.profile_name,
        snapshot,
        state.reserved_memory_bytes,
    )?;
    state.reserved_memory_bytes = requested;
    drop(state);
    Ok(MemoryReservation {
        admission,
        bytes: peak_bytes,
        active: true,
    })
}

fn validate_memory_reservation(
    peak_bytes: u64,
    profile_name: &str,
    snapshot: super::device_profiles::MemorySnapshot,
    reserved_memory_bytes: u64,
) -> Result<u64, String> {
    let required_available = peak_bytes
        .checked_add(ACTIVE_RUNTIME_HEADROOM_BYTES)
        .ok_or("resource_exhausted: Local inference available-memory requirement overflowed")?;
    if snapshot.available_bytes < required_available {
        return Err(format!(
            "resource_exhausted: {} requires {} bytes plus {} bytes application headroom, but only {} bytes are available",
            profile_name,
            peak_bytes,
            ACTIVE_RUNTIME_HEADROOM_BYTES,
            snapshot.available_bytes
        ));
    }
    let application_budget =
        super::device_profiles::application_memory_budget(snapshot.total_bytes);
    let requested = reserved_memory_bytes
        .checked_add(peak_bytes)
        .ok_or("resource_exhausted: Local inference memory reservation overflowed")?;
    if requested > application_budget {
        return Err(format!(
            "resource_exhausted: {} requires {} bytes, with {} bytes already reserved against the {} byte application budget",
            profile_name,
            peak_bytes,
            reserved_memory_bytes,
            application_budget
        ));
    }
    Ok(requested)
}

fn ensure_runtime_headroom(admission: &Admission) -> Result<(), String> {
    let snapshot = super::device_profiles::memory_snapshot()?;
    if snapshot.available_bytes < ACTIVE_RUNTIME_HEADROOM_BYTES {
        return Err(format!(
            "resource_exhausted: Local inference requires at least {ACTIVE_RUNTIME_HEADROOM_BYTES} bytes currently available"
        ));
    }
    let state = admission.state.lock().map_err(|_| "resource_busy")?;
    let application_budget =
        super::device_profiles::application_memory_budget(snapshot.total_bytes);
    if state.reserved_memory_bytes > application_budget {
        return Err(format!(
            "resource_exhausted: {} bytes are reserved against the {} byte application budget",
            state.reserved_memory_bytes, application_budget
        ));
    }
    Ok(())
}

pub(super) fn memory_pressure() -> Result<bool, String> {
    Ok(super::device_profiles::memory_snapshot()?.available_bytes < ACTIVE_RUNTIME_HEADROOM_BYTES)
}

pub(crate) async fn acquire(
    priority: Priority,
    cancellation: &CancellationToken,
    shutdown: &CancellationToken,
    deadline: Duration,
) -> Result<Permit, String> {
    if shutdown.is_cancelled() || cancellation.is_cancelled() {
        return Err("cancelled".into());
    }
    if deadline.is_zero() {
        return Err("deadline_exceeded".into());
    }
    let admission = admission();
    ensure_runtime_headroom(&admission)?;
    let (id, receiver) = {
        let mut state = admission.state.lock().map_err(|_| "resource_busy")?;
        if !state.busy && state.interactive.is_empty() && state.background.is_empty() {
            state.busy = true;
            return Ok(Permit {
                admission: admission.clone(),
                active: true,
            });
        }
        let id = state.next_id;
        state.next_id = state.next_id.wrapping_add(1);
        let (grant, receiver) = oneshot::channel();
        let waiter = Waiter {
            id,
            since: Instant::now(),
            grant,
        };
        match priority {
            Priority::Interactive => state.interactive.push_back(waiter),
            Priority::Background => state.background.push_back(waiter),
        }
        (id, receiver)
    };
    let result = tokio::select! {
        biased;
        _ = shutdown.cancelled() => Err("cancelled".to_owned()),
        _ = cancellation.cancelled() => Err("cancelled".to_owned()),
        result = tokio::time::timeout(deadline, receiver) => match result {
            Ok(Ok(permit)) => Ok(permit),
            Ok(Err(_)) => Err("worker_crashed".to_owned()),
            Err(_) => Err("deadline_exceeded".to_owned()),
        },
    };
    if result.is_err() {
        if let Ok(mut state) = admission.state.lock() {
            state.interactive.retain(|waiter| waiter.id != id);
            state.background.retain(|waiter| waiter.id != id);
        }
    }
    match result {
        Ok(permit) => {
            ensure_runtime_headroom(&admission)?;
            Ok(permit)
        }
        Err(error) => Err(error),
    }
}

impl MemoryReservation {
    pub(super) fn release(&mut self) {
        if !self.active {
            return;
        }
        if let Ok(mut state) = self.admission.state.lock() {
            state.reserved_memory_bytes = state.reserved_memory_bytes.saturating_sub(self.bytes);
        }
        self.active = false;
    }
}

impl Drop for MemoryReservation {
    fn drop(&mut self) {
        self.release();
    }
}

impl Drop for Permit {
    fn drop(&mut self) {
        if !self.active {
            return;
        }
        let Ok(mut state) = self.admission.state.lock() else {
            return;
        };
        loop {
            let aged_background = state
                .background
                .front()
                .is_some_and(|waiter| waiter.since.elapsed() >= BACKGROUND_MAX_WAIT);
            let next = if aged_background || state.interactive.is_empty() {
                state
                    .background
                    .pop_front()
                    .or_else(|| state.interactive.pop_front())
            } else {
                state.interactive.pop_front()
            };
            let Some(waiter) = next else {
                state.busy = false;
                return;
            };
            match waiter.grant.send(Permit {
                admission: self.admission.clone(),
                active: true,
            }) {
                Ok(()) => return,
                Err(mut permit) => permit.active = false,
            }
        }
    }
}

#[cfg(test)]
#[path = "admission_tests.rs"]
mod tests;
