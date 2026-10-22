#[cfg(windows)]
use super::windows_job::WindowsJob;
#[cfg(unix)]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::process::Child;

pub(crate) struct ProcessTree(Option<KillHandle>);

#[derive(Clone)]
pub(crate) enum KillHandle {
    #[cfg(unix)]
    ProcessGroup {
        id: i32,
        terminated: Arc<AtomicBool>,
    },
    #[cfg(windows)]
    Job(Arc<WindowsJob>),
}

impl KillHandle {
    pub(crate) fn terminate(&self) {
        #[cfg(unix)]
        {
            let Self::ProcessGroup { id, terminated } = self;
            if terminated.swap(true, Ordering::AcqRel) {
                return;
            }
            unsafe extern "C" {
                fn kill(pid: i32, signal: i32) -> i32;
            }
            let _ = unsafe { kill(-id, 9) };
        }
        #[cfg(windows)]
        {
            let Self::Job(job) = self;
            job.terminate();
        }
    }
}

impl ProcessTree {
    /// Takes ownership of everything `child` spawns: its process group on Unix, the job it is
    /// assigned to on Windows.
    pub(crate) fn new(child: &Child, #[cfg(windows)] job: WindowsJob) -> Result<Self, String> {
        #[cfg(windows)]
        job.assign(child)?;
        #[cfg(unix)]
        let process_group = i32::try_from(
            child
                .id()
                .ok_or("worker_crashed: worker process id unavailable")?,
        )
        .map_err(|_| "worker_crashed: worker process id exceeds platform range")?;
        Ok(Self(Some({
            #[cfg(unix)]
            {
                KillHandle::ProcessGroup {
                    id: process_group,
                    terminated: Arc::new(AtomicBool::new(false)),
                }
            }
            #[cfg(windows)]
            {
                KillHandle::Job(Arc::new(job))
            }
        })))
    }

    pub(crate) fn handle(&self) -> KillHandle {
        self.0
            .as_ref()
            .expect("process tree has a kill handle")
            .clone()
    }

    pub(crate) fn terminate(&mut self) {
        if let Some(handle) = self.0.take() {
            handle.terminate();
        }
    }
}

impl Drop for ProcessTree {
    fn drop(&mut self) {
        self.terminate();
    }
}
