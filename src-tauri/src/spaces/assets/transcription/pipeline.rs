use std::path::Path;

use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

use super::{send_partial_progress, send_progress, MediaTranscriptionProgress};
use crate::media::{
    compute_budget::ComputeBudget,
    prepared_audio::PreparedMediaAudio,
    transcript::{MediaTranscript, TranscriptAssembly},
    whisper::{self, WhisperRequest},
    workspace::AnalysisWorkspace,
};

pub struct TranscriptionPipeline<'a> {
    pub job_id: &'a str,
    pub progress: &'a Channel<MediaTranscriptionProgress>,
    pub cancellation: &'a CancellationToken,
    pub whisper_cli: &'a Path,
    pub model: &'a Path,
    pub vad_model: &'a Path,
    pub paths: &'a AnalysisWorkspace,
    pub compute_budget: ComputeBudget,
}

impl TranscriptionPipeline<'_> {
    pub fn run(&self, audio: &PreparedMediaAudio) -> Result<MediaTranscript, String> {
        send_progress(self.progress, "transcribing", 14)?;
        let duration_ms = audio.analysis.duration_ms;
        let mut assembly = TranscriptAssembly::new();
        let part = whisper::transcribe(
            WhisperRequest {
                job_id: self.job_id,
                executable: self.whisper_cli,
                model: self.model,
                vad_model: self.vad_model,
                wav: &audio.wav,
                output_base: &self.paths.transcript,
                threads: self.compute_budget.whisper_threads,
                duration_ms,
            },
            self.cancellation,
            |percent| {
                let completed = (u16::from(percent) * 76 / 100) as u8;
                send_progress(self.progress, "transcribing", 18 + completed)
            },
        )?;
        let appended = assembly.append(part, duration_ms);
        send_partial_progress(self.progress, 94, assembly.language(), appended)?;
        send_progress(self.progress, "parsing", 96)?;
        assembly.finish(duration_ms)
    }
}
