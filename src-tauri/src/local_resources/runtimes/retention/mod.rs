use std::{
    collections::BTreeSet,
    fs::{self, File},
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Debug)]
pub(crate) struct RuntimeLease(File);

pub(in crate::local_resources::runtimes) struct InstallationLease {
    lease: RuntimeLease,
    directory: PathBuf,
}

impl Drop for RuntimeLease {
    fn drop(&mut self) {
        if let Err(error) = self.0.unlock() {
            log::error!("Failed to release runtime lease: {error}");
        }
    }
}

pub(in crate::local_resources::runtimes) fn acquire(
    directory: &Path,
) -> Result<Arc<RuntimeLease>, String> {
    Ok(Arc::new(RuntimeLease(acquire_shared_file(directory)?)))
}

pub(in crate::local_resources::runtimes) fn try_acquire(
    directory: &Path,
) -> Result<Option<Arc<RuntimeLease>>, String> {
    let Some(_namespace) = try_lock_namespace(directory)? else {
        return Ok(None);
    };
    let file = open_lease(directory)?;
    match file.try_lock_shared() {
        Ok(()) => Ok(Some(Arc::new(RuntimeLease(file)))),
        Err(std::fs::TryLockError::WouldBlock) => Ok(None),
        Err(std::fs::TryLockError::Error(error)) => Err(error.to_string()),
    }
}

pub(in crate::local_resources::runtimes) fn try_acquire_exclusive(
    directory: &Path,
) -> Result<Option<InstallationLease>, String> {
    let Some(_namespace) = try_lock_namespace(directory)? else {
        return Ok(None);
    };
    let file = open_lease(directory)?;
    match file.try_lock() {
        Ok(()) => Ok(Some(InstallationLease {
            lease: RuntimeLease(file),
            directory: directory.to_path_buf(),
        })),
        Err(std::fs::TryLockError::WouldBlock) => Ok(None),
        Err(std::fs::TryLockError::Error(error)) => Err(error.to_string()),
    }
}

impl InstallationLease {
    pub(in crate::local_resources::runtimes) fn into_shared(
        self,
    ) -> Result<Arc<RuntimeLease>, String> {
        let _namespace = lock_namespace(&self.directory)?;
        self.lease.0.unlock().map_err(|error| error.to_string())?;
        self.lease
            .0
            .lock_shared()
            .map_err(|error| error.to_string())?;
        Ok(Arc::new(self.lease))
    }

    #[cfg(any(not(debug_assertions), test))]
    pub(in crate::local_resources::runtimes) fn publish_directory(
        self,
        prepared: &Path,
        destination: &Path,
    ) -> Result<Arc<RuntimeLease>, String> {
        fs::rename(prepared, destination).map_err(|error| error.to_string())?;
        self.into_shared()
    }
}

fn acquire_shared_file(directory: &Path) -> Result<File, String> {
    loop {
        let namespace = lock_namespace(directory)?;
        let file = open_lease(directory)?;
        match file.try_lock_shared() {
            Ok(()) => return Ok(file),
            Err(std::fs::TryLockError::WouldBlock) => {
                drop(file);
                drop(namespace);
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            Err(std::fs::TryLockError::Error(error)) => return Err(error.to_string()),
        }
    }
}

// Leases are outside immutable manifests and live until the last runtime/Worker reference is dropped.
pub(in crate::local_resources::runtimes) fn retire_unused(current: &Path) -> Result<(), String> {
    let _namespace = lock_namespace(current)?;
    let parent = current.parent().ok_or("Runtime directory has no parent")?;
    let mut candidates = BTreeSet::new();
    for entry in fs::read_dir(parent).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let kind = entry.file_type().map_err(|error| error.to_string())?;
        let directory = if kind.is_dir() {
            path
        } else if kind.is_file() && path.extension().is_some_and(|value| value == "lease") {
            path.with_extension("")
        } else {
            continue;
        };
        let name = directory
            .file_name()
            .ok_or("Runtime identity has no name")?
            .to_string_lossy();
        if directory == current
            || name.len() != 64
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            continue;
        }
        candidates.insert(directory);
    }
    for directory in candidates {
        if !directory.exists() && !directory.with_extension("lease").exists() {
            continue;
        }
        let lease = open_lease(&directory)?;
        match lease.try_lock() {
            Ok(()) => {
                if directory.exists() {
                    fs::remove_dir_all(&directory).map_err(|error| error.to_string())?;
                }
                drop(lease);
                fs::remove_file(directory.with_extension("lease"))
                    .map_err(|error| error.to_string())?;
            }
            Err(std::fs::TryLockError::WouldBlock) => {}
            Err(std::fs::TryLockError::Error(error)) => return Err(error.to_string()),
        }
    }
    Ok(())
}

fn lock_namespace(directory: &Path) -> Result<RuntimeLease, String> {
    let file = open_namespace(directory)?;
    file.lock().map_err(|error| error.to_string())?;
    Ok(RuntimeLease(file))
}

fn try_lock_namespace(directory: &Path) -> Result<Option<RuntimeLease>, String> {
    let file = open_namespace(directory)?;
    match file.try_lock() {
        Ok(()) => Ok(Some(RuntimeLease(file))),
        Err(std::fs::TryLockError::WouldBlock) => Ok(None),
        Err(std::fs::TryLockError::Error(error)) => Err(error.to_string()),
    }
}

fn open_namespace(directory: &Path) -> Result<File, String> {
    let parent = directory
        .parent()
        .ok_or("Runtime directory has no parent")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let file = File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(parent.join(".retention.lock"))
        .map_err(|error| error.to_string())?;
    Ok(file)
}

fn open_lease(directory: &Path) -> Result<File, String> {
    let path = directory.with_extension("lease");
    File::options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)
        .map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests;
