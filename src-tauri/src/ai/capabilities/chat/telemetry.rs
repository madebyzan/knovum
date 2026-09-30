use std::time::Instant;

use crate::ai::models::ModelExecutionTarget;

pub fn stream_request_started(target: &ModelExecutionTarget, prompt_bytes: usize) {
    log::info!(
        "AI stream request started provider={} model={} prompt_bytes={}",
        target.provider(),
        target.model(),
        prompt_bytes
    );
}

pub fn stream_opened(target: &ModelExecutionTarget, started_at: Instant) {
    log::info!(
        "AI stream opened provider={} model={} elapsed_ms={}",
        target.provider(),
        target.model(),
        started_at.elapsed().as_millis()
    );
}

pub fn stream_first_text(target: &ModelExecutionTarget, started_at: Instant) {
    log::info!(
        "AI stream first text provider={} model={} elapsed_ms={}",
        target.provider(),
        target.model(),
        started_at.elapsed().as_millis()
    );
}

pub fn stream_ended(
    target: &ModelExecutionTarget,
    chunk_count: u64,
    text_bytes: usize,
    started_at: Instant,
) {
    log::info!(
        "AI stream end provider={} model={} chunks={} text_bytes={} elapsed_ms={}",
        target.provider(),
        target.model(),
        chunk_count,
        text_bytes,
        started_at.elapsed().as_millis()
    );
}

pub fn stream_completed(target: &ModelExecutionTarget, started_at: Instant) {
    log::info!(
        "AI stream completed provider={} model={} elapsed_ms={}",
        target.provider(),
        target.model(),
        started_at.elapsed().as_millis()
    );
}
