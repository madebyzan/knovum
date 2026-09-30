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
mod tests {
    use super::*;

    #[tokio::test]
    async fn cleanup_waits_for_its_writer_without_blocking_other_runs() {
        let id = uuid::Uuid::new_v4().to_string();
        let writer = acquire(&id).await.unwrap();
        let (started, waiting) = tokio::sync::oneshot::channel();
        let cleanup = tokio::spawn(async move {
            started.send(()).unwrap();
            acquire(&id).await.unwrap()
        });
        waiting.await.unwrap();
        let other = acquire(&uuid::Uuid::new_v4().to_string()).await.unwrap();
        assert!(!cleanup.is_finished());
        drop(writer);
        cleanup.await.unwrap();
        drop(other);
    }
}
