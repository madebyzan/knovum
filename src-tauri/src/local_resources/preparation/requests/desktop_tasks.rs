use super::super::contracts::ResourcePreparationState;
use crate::desktop::tasks::contracts::{DesktopTask, DesktopTaskState};
use std::sync::OnceLock;
use tokio::sync::watch;

fn changes() -> &'static watch::Sender<u64> {
    static CHANGES: OnceLock<watch::Sender<u64>> = OnceLock::new();
    CHANGES.get_or_init(|| watch::channel(0).0)
}

pub(super) fn announce() {
    changes().send_modify(|revision| *revision = revision.wrapping_add(1));
}
pub(crate) fn subscribe() -> watch::Receiver<u64> {
    changes().subscribe()
}
pub(crate) fn snapshot() -> Result<Vec<DesktopTask>, String> {
    super::list()?
        .into_iter()
        .map(|task| {
            Ok(DesktopTask {
                id: task.job_id,
                state: match task.state {
                    ResourcePreparationState::Queued => DesktopTaskState::Queued,
                    ResourcePreparationState::Preparing => DesktopTaskState::Running,
                    ResourcePreparationState::Ready => DesktopTaskState::Succeeded,
                    ResourcePreparationState::Failed => DesktopTaskState::Failed,
                    ResourcePreparationState::Cancelled => DesktopTaskState::Cancelled,
                },
                progress: task.progress.percent,
                started_at: i64::try_from(task.started_at)
                    .map_err(|_| "invalid resource task timestamp")?,
            })
        })
        .collect()
}
