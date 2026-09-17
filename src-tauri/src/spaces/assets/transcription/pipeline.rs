use std::{path::Path, time::Instant};

use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

use super::{send_partial_progress, send_progress, MediaTranscriptionProgress};
use crate::media::{
    audio_preparation::prepare_audio,
    compute_budget::ComputeBudget,
    transcript::{MediaTranscript, TranscriptAssembly},
    whisper::{self, WhisperRequest},
    workspace::AnalysisWorkspace,
};

pub struct TranscriptionPipeline<'a> {
    pub job_id: &'a str,
    pub progress: &'a Channel<MediaTranscriptionProgress>,
    pub cancellation: &'a CancellationToken,
    pub ffmpeg: &'a Path,
    pub whisper_cli: &'a Path,
    pub model: &'a Path,
    pub vad_model: &'a Path,
    pub paths: &'a AnalysisWorkspace,
    pub compute_budget: ComputeBudget,
}

impl TranscriptionPipeline<'_> {
    pub fn run(&self, source: &Path) -> Result<MediaTranscript, String> {
        send_progress(self.progress, "transcoding", 6)?;
        let preparation_started = Instant::now();
        let prepared = prepare_audio(self.ffmpeg, source, self.paths, self.cancellation)?;
        log::info!(
            "media_transcription job={} event=audio_prepared elapsed_ms={} duration_ms={} wav_bytes={}",
            self.job_id,
            preparation_started.elapsed().as_millis(),
            prepared.duration_ms,
            std::fs::metadata(&self.paths.wav)
                .map(|value| value.len().to_string())
                .unwrap_or_else(|_| "unknown".to_owned())
        );
        send_progress(self.progress, "transcribing", 14)?;
        let mut assembly = TranscriptAssembly::new();
        let part = whisper::transcribe(
            WhisperRequest {
                job_id: self.job_id,
                executable: self.whisper_cli,
                model: self.model,
                vad_model: self.vad_model,
                wav: &self.paths.wav,
                output_base: &self.paths.transcript,
                threads: self.compute_budget.whisper_threads,
                duration_ms: prepared.duration_ms,
            },
            self.cancellation,
            |percent| {
                let completed = (u16::from(percent) * 76 / 100) as u8;
                send_progress(self.progress, "transcribing", 18 + completed)
            },
        )?;
        let appended = assembly.append(part, prepared.duration_ms);
        send_partial_progress(self.progress, 94, assembly.language(), appended)?;
        send_progress(self.progress, "parsing", 96)?;
        assembly.finish(prepared.waveform, prepared.duration_ms)
    }
}
