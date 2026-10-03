use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, OnceLock},
    time::{Duration, Instant},
};

use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy)]
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

static ADMISSION: OnceLock<Arc<Admission>> = OnceLock::new();
const BACKGROUND_MAX_WAIT: Duration = Duration::from_secs(30);

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
    let admission = ADMISSION
        .get_or_init(|| {
            Arc::new(Admission {
                state: Mutex::new(State::default()),
            })
        })
        .clone();
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
    result
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
