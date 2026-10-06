use std::{
    collections::HashMap,
    sync::{Arc, Weak},
};
use tauri::{AppHandle, Manager};
use tokio::sync::{Mutex, OwnedMutexGuard};

#[derive(Default)]
pub(crate) struct ToolInvocationConcurrency {
    intents: Mutex<HashMap<String, Weak<Mutex<()>>>>,
}

pub(crate) async fn acquire(app: &AppHandle, intent: &str) -> OwnedMutexGuard<()> {
    let state = app.state::<ToolInvocationConcurrency>();
    let mut intents = state.intents.lock().await;
    intents.retain(|_, lock| lock.strong_count() != 0);
    let lock = intents
        .get(intent)
        .and_then(Weak::upgrade)
        .unwrap_or_else(|| {
            let lock = Arc::new(Mutex::new(()));
            intents.insert(intent.to_owned(), Arc::downgrade(&lock));
            lock
        });
    drop(intents);
    lock.lock_owned().await
}
