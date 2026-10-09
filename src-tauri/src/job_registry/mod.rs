use std::{collections::HashMap, sync::Mutex};

use tokio_util::sync::CancellationToken;

pub(crate) struct JobRegistry<J = CancellationToken> {
    admission: Mutex<bool>,
    jobs: Mutex<HashMap<String, J>>,
    domain: &'static str,
}

pub(crate) struct JobAdmissionReservation {
    admission: &'static Mutex<bool>,
    domain: &'static str,
}

pub(crate) struct JobLease<J: 'static> {
    registry: &'static JobRegistry<J>,
    id: String,
}

impl Drop for JobAdmissionReservation {
    fn drop(&mut self) {
        match self.admission.lock() {
            Ok(mut reserved) => *reserved = false,
            Err(_) => log::error!("Failed to release {} maintenance admission", self.domain),
        }
    }
}

impl<J: 'static> Drop for JobLease<J> {
    fn drop(&mut self) {
        if let Err(error) = self.registry.release(&self.id) {
            log::error!("Failed to release job '{}': {error}", self.id);
        }
    }
}

impl<J: 'static> JobRegistry<J> {
    pub fn new(domain: &'static str) -> Self {
        Self {
            admission: Mutex::new(false),
            jobs: Mutex::new(HashMap::new()),
            domain,
        }
    }

    fn lock_admission(&self) -> Result<std::sync::MutexGuard<'_, bool>, String> {
        self.admission
            .lock()
            .map_err(|_| format!("{} admission lock poisoned", self.domain))
    }

    fn lock(&self) -> Result<std::sync::MutexGuard<'_, HashMap<String, J>>, String> {
        self.jobs
            .lock()
            .map_err(|_| format!("{} jobs lock poisoned", self.domain))
    }

    pub fn insert(&self, id: impl Into<String>, job: J) -> Result<(), String> {
        self.insert_unique(id, job)?;
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<J, String>
    where
        J: Clone,
    {
        self.lock()?
            .get(id)
            .cloned()
            .ok_or_else(|| format!("{} job was not found", self.domain))
    }

    pub fn remove(&self, id: &str) -> Result<Option<J>, String> {
        Ok(self.lock()?.remove(id))
    }

    pub fn reserve_maintenance(&'static self) -> Result<JobAdmissionReservation, String> {
        let mut reserved = self.lock_admission()?;
        if *reserved {
            return Err(format!(
                "{} maintenance admission is already reserved",
                self.domain
            ));
        }
        if !self.lock()?.is_empty() {
            return Err(format!("{} jobs are active", self.domain));
        }
        *reserved = true;
        Ok(JobAdmissionReservation {
            admission: &self.admission,
            domain: self.domain,
        })
    }

    fn release(&self, id: &str) -> Result<J, String> {
        self.remove(id)?
            .ok_or_else(|| format!("{} job lease was already released: {id}", self.domain))
    }

    pub fn acquire(&'static self, id: impl Into<String>, job: J) -> Result<JobLease<J>, String> {
        let id = self.insert_unique(id, job)?;
        Ok(JobLease { registry: self, id })
    }

    fn insert_unique(&self, id: impl Into<String>, job: J) -> Result<String, String> {
        let id = id.into();
        let reserved = self.lock_admission()?;
        if *reserved {
            return Err(format!("{} jobs are closed for maintenance", self.domain));
        }
        let mut jobs = self.lock()?;
        if jobs.contains_key(&id) {
            return Err(format!("{} job already exists: {id}", self.domain));
        }
        jobs.insert(id.clone(), job);
        drop(reserved);
        Ok(id)
    }
}

impl JobRegistry<CancellationToken> {
    pub fn acquire_cancellable(
        &'static self,
        id: impl Into<String>,
    ) -> Result<(CancellationToken, JobLease<CancellationToken>), String> {
        let token = CancellationToken::new();
        let lease = self.acquire(id, token.clone())?;
        Ok((token, lease))
    }

    pub fn cancel(&self, id: &str) -> Result<bool, String> {
        if let Some(token) = self.lock()?.get(id) {
            token.cancel();
            return Ok(true);
        }
        Ok(false)
    }
}

#[cfg(test)]
mod tests;
