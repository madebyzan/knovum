use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::time::{SystemTime, UNIX_EPOCH};

use ort::session::RunOptions;

#[derive(Clone)]
pub(crate) struct Cancellation {
    cancelled: Arc<AtomicBool>,
    run: Arc<Mutex<Option<Arc<RunOptions>>>>,
    deadline_ms: u64,
}

impl Cancellation {
    pub(crate) fn new(deadline_ms: u64) -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
            run: Arc::new(Mutex::new(None)),
            deadline_ms,
        }
    }

    pub(crate) fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
        if let Ok(run) = self.run.lock() {
            if let Some(run) = run.as_ref() {
                let _ = run.terminate();
            }
        }
    }

    pub(crate) fn check(&self) -> Result<(), String> {
        if self.cancelled.load(Ordering::Acquire) {
            return Err("cancelled".into());
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_millis();
        if now >= u128::from(self.deadline_ms) {
            return Err("deadline_exceeded".into());
        }
        Ok(())
    }

    pub(crate) fn run_options(&self) -> Result<Arc<RunOptions>, String> {
        let options =
            Arc::new(RunOptions::new().map_err(|error| format!("resource_exhausted: {error}"))?);
        let mut run = self
            .run
            .lock()
            .map_err(|_| "worker_crashed: cancellation lock poisoned")?;
        *run = Some(options.clone());
        self.check()?;
        Ok(options)
    }
}

#[derive(Default)]
pub(crate) struct ActiveRequest(pub(crate) Mutex<Option<(String, u64, Cancellation)>>);

impl ActiveRequest {
    pub(crate) fn admit(
        &self,
        request_id: &str,
        epoch: u64,
        deadline: u64,
    ) -> Result<Cancellation, String> {
        let mut active = self
            .0
            .lock()
            .map_err(|_| "worker_crashed: admission lock poisoned")?;
        if active.is_some() {
            return Err("resource_exhausted: worker already has an active request".into());
        }
        let cancel = Cancellation::new(deadline);
        cancel.check()?;
        *active = Some((request_id.into(), epoch, cancel.clone()));
        Ok(cancel)
    }

    pub(crate) fn cancel(&self, request_id: &str, epoch: u64) -> Result<bool, String> {
        let active = self
            .0
            .lock()
            .map_err(|_| "worker_crashed: admission lock poisoned")?;
        if let Some((current, attempt, cancellation)) = active
            .as_ref()
            .filter(|(current, attempt, _)| current == request_id && *attempt == epoch)
        {
            let _ = (current, attempt);
            cancellation.cancel();
            return Ok(true);
        }
        Ok(false)
    }

    pub(crate) fn finish(&self) -> Result<(), String> {
        self.0
            .lock()
            .map_err(|_| "worker_crashed: admission lock poisoned")?
            .take();
        Ok(())
    }
}

#[cfg(test)]
#[path = "tests/cancellation.rs"]
mod tests;
