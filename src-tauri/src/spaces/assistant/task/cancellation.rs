use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use tokio_util::sync::CancellationToken;

static ACTIVE: OnceLock<Mutex<HashMap<String, CancellationToken>>> = OnceLock::new();

pub(super) fn register(run_id: &str) -> Result<CancellationToken, String> {
    let token = CancellationToken::new();
    active()
        .lock()
        .map_err(|_| "assistant-cancellation-registry-unavailable")?
        .insert(run_id.to_owned(), token.clone());
    Ok(token)
}

pub(super) fn cancel(run_id: &str) -> Result<(), String> {
    if let Some(token) = active()
        .lock()
        .map_err(|_| "assistant-cancellation-registry-unavailable")?
        .get(run_id)
    {
        token.cancel();
    }
    Ok(())
}

pub(super) fn remove(run_id: &str) {
    if let Ok(mut active) = active().lock() {
        active.remove(run_id);
    }
}

fn active() -> &'static Mutex<HashMap<String, CancellationToken>> {
    ACTIVE.get_or_init(|| Mutex::new(HashMap::new()))
}
