use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex, OnceLock,
};

use tauri::{AppHandle, Runtime};
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

use super::{coordinator, ToolPreparationProgress};

#[derive(Clone)]
enum State {
    Preparing(ToolPreparationProgress),
    Ready,
    Failed(Arc<str>),
}

struct Installation {
    cancellation: CancellationToken,
    state: watch::Sender<State>,
    waiters: AtomicUsize,
}

impl Installation {
    fn new() -> Arc<Self> {
        let (state, _) = watch::channel(State::Preparing(ToolPreparationProgress {
            stage: super::super::preparation::contracts::ResourcePreparationStage::Queued,
            downloaded: 0,
            total: None,
        }));
        Arc::new(Self {
            cancellation: CancellationToken::new(),
            state,
            waiters: AtomicUsize::new(1),
        })
    }
}

struct Registry {
    active: Mutex<Option<Arc<Installation>>>,
}

static REGISTRY: OnceLock<Arc<Registry>> = OnceLock::new();

pub(super) struct Waiter {
    registry: Arc<Registry>,
    installation: Arc<Installation>,
    state: watch::Receiver<State>,
}

impl Waiter {
    fn new(registry: Arc<Registry>, installation: Arc<Installation>) -> Self {
        let state = installation.state.subscribe();
        Self {
            registry,
            installation,
            state,
        }
    }

    pub(super) async fn wait(
        &mut self,
        request_cancellation: &CancellationToken,
        progress: &mut impl FnMut(ToolPreparationProgress),
    ) -> Result<(), String> {
        loop {
            if request_cancellation.is_cancelled() {
                return Err(cancelled());
            }
            match self.state.borrow_and_update().clone() {
                State::Preparing(update) => progress(update),
                State::Ready => return Ok(()),
                State::Failed(error) => return Err(error.to_string()),
            }
            tokio::select! {
                biased;
                _ = request_cancellation.cancelled() => {
                    return Err(cancelled());
                }
                changed = self.state.changed() => {
                    if changed.is_err() {
                        return Err("Media import tool preparation ended without a terminal state".to_string());
                    }
                }
            }
        }
    }
}

impl Drop for Waiter {
    fn drop(&mut self) {
        let Ok(active) = self.registry.active.lock() else {
            return;
        };
        if active
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, &self.installation))
            && self.installation.waiters.fetch_sub(1, Ordering::AcqRel) == 1
        {
            self.installation.cancellation.cancel();
        }
    }
}

pub(super) async fn join<R: Runtime>(
    app: &AppHandle<R>,
    request_cancellation: &CancellationToken,
) -> Result<Waiter, String> {
    let registry = registry();
    loop {
        if request_cancellation.is_cancelled() {
            return Err(cancelled());
        }
        let retiring = {
            let mut active = registry
                .active
                .lock()
                .map_err(|_| "Media import tool preparation lock poisoned".to_string())?;
            match active.as_ref() {
                Some(current) if current.cancellation.is_cancelled() => {
                    Some(current.state.subscribe())
                }
                Some(current) => {
                    current.waiters.fetch_add(1, Ordering::AcqRel);
                    return Ok(Waiter::new(Arc::clone(&registry), Arc::clone(current)));
                }
                None => {
                    let installation = Installation::new();
                    *active = Some(Arc::clone(&installation));
                    start(
                        app.clone(),
                        Arc::clone(&registry),
                        Arc::clone(&installation),
                    );
                    return Ok(Waiter::new(Arc::clone(&registry), installation));
                }
            }
        };
        let mut retiring = retiring.expect("cancelled installation must have a state receiver");
        tokio::select! {
            biased;
            _ = request_cancellation.cancelled() => {
                return Err(cancelled());
            }
            _ = retiring.changed() => {}
        }
    }
}

fn start<R: Runtime>(app: AppHandle<R>, registry: Arc<Registry>, installation: Arc<Installation>) {
    tauri::async_runtime::spawn(async move {
        let installation_outcome =
            coordinator::install_missing(&app, &installation.cancellation, &mut |progress| {
                installation.state.send_replace(State::Preparing(progress));
            })
            .await;
        if let Ok(mut active) = registry.active.lock() {
            if active
                .as_ref()
                .is_some_and(|current| Arc::ptr_eq(current, &installation))
            {
                *active = None;
            }
        }
        installation.state.send_replace(match installation_outcome {
            Ok(()) => State::Ready,
            Err(error) => State::Failed(error.into()),
        });
    });
}

fn registry() -> Arc<Registry> {
    Arc::clone(REGISTRY.get_or_init(|| {
        Arc::new(Registry {
            active: Mutex::new(None),
        })
    }))
}

fn cancelled() -> String {
    super::super::preparation::RESOURCE_PREPARATION_CANCELLED.to_string()
}

#[cfg(test)]
mod tests;
