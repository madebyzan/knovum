use std::{
    path::Path,
    time::{Duration, Instant},
};

use serde::{de::DeserializeOwned, Serialize};
use tauri::{AppHandle, Runtime};
use tokio::sync::OwnedMutexGuard;
use tokio_util::sync::CancellationToken;

use super::{
    admission,
    process::{Worker, WorkerSpecification},
    protocol::EventHandler,
    WorkerSlot,
};
use crate::{ai::contracts::runtime::RuntimeCancelParams, local_resources::runtimes::inference};

/// Grace a worker is given to abandon an interrupted request before it is killed.
const CANCEL_GRACE: Duration = Duration::from_secs(5);

/// What one request is: the identity the worker must echo and the budget that bounds it.
pub(super) struct RequestIdentity<'a> {
    pub(super) request_id: &'a str,
    pub(super) attempt_epoch: u64,
    pub(super) budget_ms: u64,
}

/// The single inference slot, held on a worker that serves one specification.
///
/// Every capability reaches a worker through this lease, so the admission order, the compatibility
/// check that decides whether the resident worker can be reused, and the reclamation of an idle
/// worker exist once rather than once per capability.
pub(super) struct WorkerLease {
    guard: OwnedMutexGuard<Option<Worker>>,
    _permit: admission::Permit,
    slot: WorkerSlot,
}

impl Drop for WorkerLease {
    fn drop(&mut self) {
        super::reclaim_when_idle(self.slot.clone());
    }
}

impl WorkerLease {
    /// Holds the slot on a worker that serves `specification`, starting one when the resident
    /// worker serves something else.
    pub(super) async fn acquire<R: Runtime>(
        app: &AppHandle<R>,
        priority: admission::Priority,
        specification: WorkerSpecification,
        cancellation: &CancellationToken,
    ) -> Result<Self, String> {
        let exit = super::exit_token();
        let runtime = inference::resolve(app)?;
        super::prepare_worker_for_admission(priority, |worker| {
            worker.serves(&runtime.digest, &specification)
        })
        .await?;
        let permit = admission::acquire(priority, cancellation, exit).await?;
        let slot = super::worker_slot();
        let mut guard = tokio::select! {
            biased;
            _ = exit.cancelled() => return Err("cancelled".into()),
            _ = cancellation.cancelled() => return Err("cancelled".into()),
            guard = slot.clone().lock_owned() => guard,
        };
        if let Some(worker) = guard.as_mut() {
            if !worker.serves(&runtime.digest, &specification) {
                worker.terminate().await;
                *guard = None;
            }
        }
        if guard.is_none() {
            *guard = Some(
                Worker::start(
                    &runtime,
                    &super::inference_root(app)?,
                    specification,
                    cancellation,
                    exit,
                )
                .await?,
            );
        }
        Ok(Self {
            guard,
            _permit: permit,
            slot,
        })
    }

    /// The private directory this worker exchanges artifacts in.
    pub(super) fn work_directory(&self) -> Result<&Path, String> {
        Ok(self
            .guard
            .as_ref()
            .ok_or("worker_crashed")?
            .work_directory())
    }

    /// Discards the worker this lease holds, so the next lease starts a fresh one.
    ///
    /// A response that fails the caller's validation leaves a worker whose behaviour is no longer
    /// described by this protocol, and such a worker is never handed to another request.
    pub(super) async fn discard(&mut self) {
        if let Some(worker) = self.guard.as_mut() {
            worker.terminate().await;
        }
        *self.guard = None;
    }

    /// Runs one request that answers with a single response.
    pub(super) async fn request<O: DeserializeOwned>(
        &mut self,
        identity: &RequestIdentity<'_>,
        params: &impl Serialize,
        cancellation: &CancellationToken,
    ) -> Result<O, String> {
        self.request_with_events(
            identity,
            params,
            cancellation,
            &mut super::protocol::reject_events(),
        )
        .await
    }

    /// Runs one request whose worker may publish progress or stream chunks before it responds.
    ///
    /// The method is the one method this worker's frozen capability answers, the deadline starts
    /// when the request is dispatched, and a worker that is cancelled, times out or answers
    /// invalidly is given its abandonment grace and then terminated. Every capability shares this
    /// one lifecycle, so none of them can leave a half-cancelled worker in the slot.
    pub(super) async fn request_with_events<O: DeserializeOwned>(
        &mut self,
        identity: &RequestIdentity<'_>,
        params: &impl Serialize,
        cancellation: &CancellationToken,
        on_event: &mut EventHandler<'_>,
    ) -> Result<O, String> {
        let exit = super::exit_token();
        let worker = self.guard.as_mut().ok_or("worker_crashed")?;
        let method = worker.capability.method();
        let dispatched = worker
            .send_request(
                identity.request_id,
                method,
                params,
                identity.budget_ms,
                cancellation,
                exit,
            )
            .await;
        let deadline = match dispatched {
            Ok(deadline) => deadline,
            // A request that was never dispatched already terminated its worker.
            Err(error) => {
                *self.guard = None;
                return Err(error);
            }
        };
        let worker = self.guard.as_mut().ok_or("worker_crashed")?;
        let (response, interrupted) = tokio::select! {
            biased;
            _ = exit.cancelled() => (Err("cancelled".into()), true),
            _ = cancellation.cancelled() => (Err("cancelled".into()), true),
            response = tokio::time::timeout_at(deadline, worker.protocol.read_response_with_events(identity.request_id, on_event)) => match response {
                Ok(response) => (response, false),
                Err(_) => (Err("deadline_exceeded".into()), true),
            },
        };
        if response.is_err() {
            if interrupted {
                abandon(worker, identity).await;
            }
            worker.terminate().await;
            *self.guard = None;
        } else {
            worker.last_used = Instant::now();
        }
        response
    }
}

/// Asks a worker to abandon an interrupted request and waits out its grace for a final answer.
///
/// The answer is discarded and the worker is terminated either way, so notifications that arrive
/// during the grace are read and dropped rather than published to a caller that has already gone.
async fn abandon(worker: &mut Worker, identity: &RequestIdentity<'_>) {
    let _ = tokio::time::timeout(CANCEL_GRACE, async {
        worker
            .protocol
            .notify(
                "runtime.cancel",
                &RuntimeCancelParams {
                    target_request_id: identity.request_id.into(),
                    attempt_epoch: identity.attempt_epoch,
                },
            )
            .await?;
        while worker.protocol.read().await?.get("method").is_some() {}
        Ok::<(), String>(())
    })
    .await;
}
