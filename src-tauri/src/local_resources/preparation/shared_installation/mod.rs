use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};

use std::future::Future;
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

use super::RESOURCE_PREPARATION_CANCELLED;

#[derive(Clone)]
enum State<P> {
    Preparing(P),
    Ready,
    Failed(Arc<str>),
}

pub(crate) struct Installation<P> {
    pub(crate) cancellation: CancellationToken,
    state: watch::Sender<State<P>>,
    waiters: AtomicUsize,
}

impl<P: Clone> Installation<P> {
    fn new(initial: P) -> Arc<Self> {
        let (state, _) = watch::channel(State::Preparing(initial));
        Arc::new(Self {
            cancellation: CancellationToken::new(),
            state,
            waiters: AtomicUsize::new(1),
        })
    }
}

pub(crate) struct Registry<P> {
    active: Mutex<Option<Arc<Installation<P>>>>,
}

pub(crate) struct Waiter<P> {
    registry: Arc<Registry<P>>,
    installation: Arc<Installation<P>>,
    state: watch::Receiver<State<P>>,
}

impl<P: Clone> Waiter<P> {
    fn new(registry: Arc<Registry<P>>, installation: Arc<Installation<P>>) -> Self {
        let state = installation.state.subscribe();
        Self {
            registry,
            installation,
            state,
        }
    }

    pub(crate) async fn wait(
        &mut self,
        request_cancellation: &CancellationToken,
        progress: &mut impl FnMut(P),
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
                        return Err("Shared resource preparation ended without a terminal state".to_string());
                    }
                }
            }
        }
    }
}

impl<P> Drop for Waiter<P> {
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

impl<P: Clone + Send + Sync + 'static> Registry<P> {
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            active: Mutex::new(None),
        })
    }

    pub(crate) async fn join<F, Fut>(
        self: &Arc<Self>,
        request_cancellation: &CancellationToken,
        initial: P,
        install: F,
    ) -> Result<Waiter<P>, String>
    where
        F: FnOnce(Arc<Installation<P>>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), String>> + Send + 'static,
    {
        let registry = self;
        loop {
            if request_cancellation.is_cancelled() {
                return Err(cancelled());
            }
            let retiring = {
                let mut active = registry
                    .active
                    .lock()
                    .map_err(|_| "Shared resource preparation lock poisoned".to_string())?;
                match active.as_ref() {
                    Some(current) if current.cancellation.is_cancelled() => {
                        Some(current.state.subscribe())
                    }
                    Some(current) => {
                        current.waiters.fetch_add(1, Ordering::AcqRel);
                        return Ok(Waiter::new(Arc::clone(registry), Arc::clone(current)));
                    }
                    None => {
                        let installation = Installation::new(initial);
                        *active = Some(Arc::clone(&installation));
                        start(Arc::clone(registry), Arc::clone(&installation), install);
                        return Ok(Waiter::new(Arc::clone(registry), installation));
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
}

impl<P: Clone> Installation<P> {
    pub(crate) fn publish(&self, progress: P) {
        self.state.send_replace(State::Preparing(progress));
    }
}

fn start<P, F, Fut>(registry: Arc<Registry<P>>, installation: Arc<Installation<P>>, install: F)
where
    P: Clone + Send + Sync + 'static,
    F: FnOnce(Arc<Installation<P>>) -> Fut + Send + 'static,
    Fut: Future<Output = Result<(), String>> + Send + 'static,
{
    tauri::async_runtime::spawn(async move {
        let installation_outcome = install(Arc::clone(&installation)).await;
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

fn cancelled() -> String {
    RESOURCE_PREPARATION_CANCELLED.to_string()
}

#[cfg(test)]
mod tests;
