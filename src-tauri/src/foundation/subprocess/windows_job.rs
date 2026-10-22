use tokio::process::Child;
use windows::Win32::{
    Foundation::{CloseHandle, HANDLE},
    System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, JobObjectExtendedLimitInformation,
        SetInformationJobObject, TerminateJobObject, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    },
};

pub(crate) struct WindowsJob(HANDLE);

// A job handle may be used from any thread, and this type owns the only handle copy.
unsafe impl Send for WindowsJob {}
unsafe impl Sync for WindowsJob {}

impl WindowsJob {
    pub(crate) fn new() -> Result<Self, String> {
        let job = unsafe { CreateJobObjectW(None, None) }
            .map_err(|error| format!("worker_crashed: failed to create process job: {error}"))?;
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        if let Err(error) = unsafe {
            SetInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                &limits as *const _ as _,
                std::mem::size_of_val(&limits) as u32,
            )
        } {
            let _ = unsafe { CloseHandle(job) };
            return Err(format!(
                "worker_crashed: failed to configure process job: {error}"
            ));
        }
        Ok(Self(job))
    }

    pub(super) fn assign(&self, child: &Child) -> Result<(), String> {
        let process = child
            .raw_handle()
            .ok_or("worker_crashed: worker exited before it could be contained")?;
        unsafe { AssignProcessToJobObject(self.0, HANDLE(process)) }
            .map_err(|error| format!("worker_crashed: failed to contain worker process: {error}"))
    }

    pub(crate) fn terminate(&self) {
        let _ = unsafe { TerminateJobObject(self.0, 1) };
    }
}

impl Drop for WindowsJob {
    fn drop(&mut self) {
        let _ = unsafe { CloseHandle(self.0) };
    }
}
