//! Records transcription engine telemetry without domain state.

use std::{fs, time::Instant};

use super::WhisperRequest;

pub(crate) struct WhisperTelemetry<'a> {
    job_id: &'a str,
    duration_ms: u64,
    started: Instant,
    next_milestone: u8,
}

impl<'a> WhisperTelemetry<'a> {
    pub(crate) fn new(request: &'a WhisperRequest<'_>) -> Self {
        let model_bytes = fs::metadata(request.model)
            .map(|value| value.len().to_string())
            .unwrap_or_else(|_| "unknown".to_owned());
        let input_bytes = fs::metadata(request.wav)
            .map(|value| value.len().to_string())
            .unwrap_or_else(|_| "unknown".to_owned());
        log::info!(
            "media_transcription job={} event=whisper_started duration_ms={} model_bytes={} input_bytes={} threads={} backend={}",
            request.job_id,
            request.duration_ms,
            model_bytes,
            input_bytes,
            request.threads,
            backend_name()
        );
        Self {
            job_id: request.job_id,
            duration_ms: request.duration_ms,
            started: Instant::now(),
            next_milestone: 0,
        }
    }

    pub(crate) fn progress(&mut self, percent: u8) {
        if percent < self.next_milestone {
            return;
        }
        log::info!(
            "media_transcription job={} event=whisper_progress percent={} elapsed_ms={}",
            self.job_id,
            percent,
            self.started.elapsed().as_millis()
        );
        self.next_milestone = percent.saturating_add(10).min(100);
    }

    pub(crate) fn finish(&self, success: bool) {
        let elapsed_ms = self.started.elapsed().as_millis();
        let realtime_factor = elapsed_ms as f64 / self.duration_ms.max(1) as f64;
        log::info!(
            "media_transcription job={} event=whisper_finished success={} elapsed_ms={} realtime_factor={:.3}",
            self.job_id,
            success,
            elapsed_ms,
            realtime_factor
        );
    }
}

pub(crate) const fn uses_metal_backend() -> bool {
    cfg!(all(target_os = "macos", target_arch = "aarch64"))
}

const fn backend_name() -> &'static str {
    if uses_metal_backend() {
        "metal"
    } else {
        "cpu"
    }
}
