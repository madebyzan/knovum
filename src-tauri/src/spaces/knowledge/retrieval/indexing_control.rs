use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};
use tokio_util::sync::CancellationToken;

use crate::foundation::job_registry::{JobLease, JobRegistry};

/// Indexing a Space is identified by the Space, not by the press that asked for it.
///
/// Two passes over one Space are never useful: they read the same unindexed chunks, spend the one
/// local embedding worker twice on them, and — when neither can reuse a generation — each start a
/// rebuild of which only one can ever be activated. So a Space has a turn, and every pass takes it.
///
/// Cancellation is only ever observed between batches in `generation_build::run` — never inside a
/// vector write and never between a completed build and `generation::activate`. A cancelled pass
/// therefore leaves committed markers for the chunks it did encode and the previous generation
/// still answering searches; the next pass resumes from exactly there.
pub(crate) const BUSY: &str = "knowledge-indexing-busy";

/// Why a caller wants a Space indexed, which is what decides the answer when a pass already holds
/// the Space's turn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Admission {
    /// Somebody asked for this Space to be indexed. A running pass is already doing that work, so
    /// the request is declined with [`BUSY`] instead of duplicated, and the pass it does start can
    /// be cancelled by Space id.
    Requested,
    /// Work that cannot continue until the Space is searchable. It waits for the running pass and
    /// then fills whatever its own scope still needs, under the cancellation of the Run that
    /// needs it — a person cancels that Run, not the indexing inside it.
    Required,
}

/// The right to index one Space, held for as long as the pass runs.
pub(crate) struct IndexingPass {
    cancellation: CancellationToken,
    _turn: OwnedMutexGuard<()>,
    _cancellable: Option<JobLease<CancellationToken>>,
}

impl IndexingPass {
    pub(crate) fn cancellation(&self) -> &CancellationToken {
        &self.cancellation
    }
}

/// One turn per Space, kept for the life of the process: a handful of bytes per Space in the
/// library, against which a `HashMap` entry removed on release would race with the next caller
/// that is already waiting on it.
fn turns() -> &'static Mutex<HashMap<String, Arc<AsyncMutex<()>>>> {
    static TURNS: OnceLock<Mutex<HashMap<String, Arc<AsyncMutex<()>>>>> = OnceLock::new();
    TURNS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// The tokens a person can pull, keyed by Space id — only passes somebody asked for are in here.
fn cancellations() -> &'static JobRegistry {
    static CANCELLATIONS: OnceLock<JobRegistry> = OnceLock::new();
    CANCELLATIONS.get_or_init(|| JobRegistry::new("Knowledge indexing"))
}

fn turn_for(space_id: &str) -> Result<Arc<AsyncMutex<()>>, String> {
    let mut turns = turns()
        .lock()
        .map_err(|_| "Knowledge indexing turn lock poisoned".to_owned())?;
    Ok(Arc::clone(turns.entry(space_id.to_owned()).or_default()))
}

pub(crate) async fn begin(
    space_id: &str,
    admission: Admission,
    cancellation: CancellationToken,
) -> Result<IndexingPass, String> {
    let turn = turn_for(space_id)?;
    let turn = match admission {
        Admission::Requested => turn.try_lock_owned().map_err(|_| BUSY.to_owned())?,
        // Waiting for a turn is still waiting: a Run cancelled while it queues stops here rather
        // than at the end of a pass it no longer needs.
        Admission::Required => tokio::select! {
            guard = turn.lock_owned() => guard,
            () = cancellation.cancelled() => return Err(super::rebuild::CANCELLED.to_owned()),
        },
    };
    let cancellable = match admission {
        Admission::Requested => {
            Some(cancellations().acquire(space_id.to_owned(), cancellation.clone())?)
        }
        Admission::Required => None,
    };
    Ok(IndexingPass {
        cancellation,
        _turn: turn,
        _cancellable: cancellable,
    })
}

pub(crate) fn cancel(space_id: &str) -> Result<bool, String> {
    cancellations().cancel(space_id)
}

/// Whether a pass holds this Space's turn, whichever admission it came in under.
pub(crate) fn is_running(space_id: &str) -> Result<bool, String> {
    let turns = turns()
        .lock()
        .map_err(|_| "Knowledge indexing turn lock poisoned".to_owned())?;
    Ok(turns
        .get(space_id)
        .is_some_and(|turn| turn.try_lock().is_err()))
}
