use crate::cancellation::Cancellation;
use knovum_inference_contracts::runtime::RuntimeAckChunkParams;
use std::{
    collections::BTreeMap,
    sync::{Condvar, Mutex},
    time::Duration,
};

struct State {
    pending: BTreeMap<u64, u64>,
    bytes: u64,
    failed: bool,
}

pub(crate) struct Credits {
    request: String,
    epoch: u64,
    state: Mutex<State>,
    wake: Condvar,
}

impl Credits {
    pub(crate) fn new(request: &str, epoch: u64) -> Self {
        Self {
            request: request.into(),
            epoch,
            state: Mutex::new(State {
                pending: BTreeMap::new(),
                bytes: 0,
                failed: false,
            }),
            wake: Condvar::new(),
        }
    }
    pub(crate) fn reserve(
        &self,
        seq: u64,
        bytes: u64,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        if bytes == 0 || bytes > 16 * 1024 * 1024 {
            return Err("resource_exhausted: synthesis chunk exceeds its credit window".into());
        }
        let mut state = self
            .state
            .lock()
            .map_err(|_| "worker_crashed: synthesis credit lock poisoned")?;
        loop {
            cancel.check()?;
            if state.failed {
                return Err("invalid_output: synthesis chunk acknowledgement differs".into());
            }
            if state.pending.len() < 8 && state.bytes + bytes <= 16 * 1024 * 1024 {
                break;
            }
            state = self
                .wake
                .wait_timeout(state, Duration::from_millis(100))
                .map_err(|_| "worker_crashed: synthesis credit lock poisoned")?
                .0;
        }
        if seq == 0 || state.pending.contains_key(&seq) {
            return Err("invalid_output: synthesis chunk sequence was reused".into());
        }
        state.pending.insert(seq, bytes);
        state.bytes += bytes;
        Ok(())
    }
    pub(crate) fn acknowledge(&self, params: &RuntimeAckChunkParams) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "worker_crashed: synthesis credit lock poisoned")?;
        if params.request_id != self.request
            || params.attempt_epoch != self.epoch
            || state.pending.get(&params.seq) != Some(&params.consumed_bytes)
        {
            state.failed = true;
            self.wake.notify_all();
            return Err("invalid_input: synthesis acknowledgement identity or size differs".into());
        }
        state.pending.remove(&params.seq);
        state.bytes -= params.consumed_bytes;
        self.wake.notify_all();
        Ok(())
    }
    pub(crate) fn wait(&self, cancel: &Cancellation) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "worker_crashed: synthesis credit lock poisoned")?;
        loop {
            cancel.check()?;
            if state.failed {
                return Err("invalid_output: synthesis chunk acknowledgement differs".into());
            }
            if state.pending.is_empty() {
                return Ok(());
            }
            state = self
                .wake
                .wait_timeout(state, Duration::from_millis(100))
                .map_err(|_| "worker_crashed: synthesis credit lock poisoned")?
                .0;
        }
    }
}
