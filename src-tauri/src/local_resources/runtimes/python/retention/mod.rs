use std::{
    fs::{self, File},
    path::Path,
    sync::Arc,
};

#[derive(Debug)]
pub(crate) struct RuntimeLease(File);

impl Drop for RuntimeLease {
    fn drop(&mut self) {
        if let Err(error) = self.0.unlock() {
            log::error!("Failed to release Python runtime lease: {error}");
        }
    }
}

pub(super) fn acquire(directory: &Path) -> Result<Arc<RuntimeLease>, String> {
    let file = open_lease(directory)?;
    file.lock_shared().map_err(|error| error.to_string())?;
    Ok(Arc::new(RuntimeLease(file)))
}

// Leases are outside immutable manifests and live until the last runtime/Worker reference is dropped.
pub(super) fn retire_unused(current: &Path) -> Result<(), String> {
    let parent = current.parent().ok_or("Runtime directory has no parent")?;
    for entry in fs::read_dir(parent).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if entry.path() == current
            || !entry
                .file_type()
                .map_err(|error| error.to_string())?
                .is_dir()
            || name.len() != 64
            || !name
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            continue;
        }
        let lease = open_lease(&entry.path())?;
        match lease.try_lock() {
            Ok(()) => fs::remove_dir_all(entry.path()).map_err(|error| error.to_string())?,
            Err(std::fs::TryLockError::WouldBlock) => {}
            Err(std::fs::TryLockError::Error(error)) => return Err(error.to_string()),
        }
    }
    Ok(())
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
