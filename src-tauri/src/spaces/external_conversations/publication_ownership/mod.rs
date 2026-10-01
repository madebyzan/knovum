use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex, Weak},
};

use tokio::sync::{Mutex as AsyncMutex, OwnedMutexGuard};

static OWNERS: LazyLock<Mutex<HashMap<String, Weak<AsyncMutex<()>>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// A writer retains ownership until its file writes and rollback have settled.
/// Cancellation and deletion acquire the same ownership before removing its files.
pub(super) async fn acquire(run_id: &str) -> Result<OwnedMutexGuard<()>, String> {
    let owner = {
        let mut owners = OWNERS
            .lock()
            .map_err(|_| "Agent publication ownership is unavailable")?;
        owners.retain(|_, owner| owner.strong_count() > 0);
        match owners.get(run_id).and_then(Weak::upgrade) {
            Some(owner) => owner,
            None => {
                let owner = Arc::new(AsyncMutex::new(()));
                owners.insert(run_id.to_owned(), Arc::downgrade(&owner));
                owner
            }
        }
    };
    Ok(owner.lock_owned().await)
}

#[cfg(test)]
mod tests;
