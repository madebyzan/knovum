use std::{
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

use tokio_util::sync::CancellationToken;

static ACTIVE: OnceLock<Mutex<HashMap<String, CancellationToken>>> = OnceLock::new();
static CHANGES: OnceLock<tokio::sync::watch::Sender<u64>> = OnceLock::new();

pub(super) fn register(run_id: &str) -> Result<CancellationToken, String> {
    let token = CancellationToken::new();
    let mut active = active()
        .lock()
        .map_err(|_| "builtin-agent-cancellation-registry-unavailable")?;
    if active.contains_key(run_id) {
        return Err("builtin-agent-execution-already-active".into());
    }
    active.insert(run_id.to_owned(), token.clone());
    Ok(token)
}

pub(super) fn cancel(run_id: &str) -> Result<(), String> {
    if let Some(token) = active()
        .lock()
        .map_err(|_| "builtin-agent-cancellation-registry-unavailable")?
        .get(run_id)
    {
        token.cancel();
    }
    Ok(())
}

pub(super) fn remove(run_id: &str) {
    if let Ok(mut active) = active().lock() {
        active.remove(run_id);
        changes().send_modify(|revision| *revision += 1);
    }
}

fn changes() -> &'static tokio::sync::watch::Sender<u64> {
    CHANGES.get_or_init(|| tokio::sync::watch::channel(0).0)
}

pub(super) async fn wait_idle(run_id: &str) -> Result<(), String> {
    let mut updates = changes().subscribe();
    loop {
        if !active()
            .lock()
            .map_err(|_| "builtin-agent-cancellation-registry-unavailable")?
            .contains_key(run_id)
        {
            return Ok(());
        }
        updates
            .changed()
            .await
            .map_err(|_| "builtin-agent-cancellation-registry-closed")?;
    }
}

fn active() -> &'static Mutex<HashMap<String, CancellationToken>> {
    ACTIVE.get_or_init(|| Mutex::new(HashMap::new()))
}
