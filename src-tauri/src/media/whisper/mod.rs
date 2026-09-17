use std::{
    path::Path,
    process::{Command, Stdio},
};

use tokio_util::sync::CancellationToken;

mod output;
mod telemetry;

use super::{
    subprocess::{diagnostic_error, parse_progress, wait_with_stderr},
    subtitle_timing::SubtitleCue,
    TRANSCRIPTION_CANCELLED,
};
use telemetry::{uses_metal_backend, WhisperTelemetry};

pub(crate) struct WhisperOutput {
    pub language: String,
    pub segments: Vec<WhisperSegment>,
    pub subtitle_cues: Vec<SubtitleCue>,
}

pub struct WhisperRequest<'a> {
    pub job_id: &'a str,
    pub executable: &'a Path,
    pub model: &'a Path,
    pub vad_model: &'a Path,
    pub wav: &'a Path,
    pub output_base: &'a Path,
    pub threads: usize,
    pub duration_ms: u64,
}

pub struct WhisperSegment {
    pub start: f64,
    pub end: f64,
    pub text: String,
}

pub fn transcribe(
    request: WhisperRequest<'_>,
    cancellation: &CancellationToken,
    mut on_progress: impl FnMut(u8) -> Result<(), String>,
) -> Result<WhisperOutput, String> {
    let mut command = command(&request);
    let mut telemetry = WhisperTelemetry::new(&request);
    let child = command
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to launch whisper.cpp CLI: {error}"))?;
    let output = wait_with_stderr(child, cancellation, TRANSCRIPTION_CANCELLED, |line| {
        if let Some(percent) = parse_progress(line, "progress =") {
            telemetry.progress(percent);
            on_progress(percent)?;
        }
        Ok(())
    })?;
    telemetry.finish(output.status.success());
    if !output.status.success() {
        return Err(diagnostic_error(
            &output.diagnostics,
            "whisper.cpp could not transcribe media",
        ));
    }
    output::read(&request.output_base.with_extension("json"))
}

fn command(request: &WhisperRequest<'_>) -> Command {
    let mut command = Command::new(request.executable);
    command
        .args(["-m"])
        .arg(request.model)
        .args(["-f"])
        .arg(request.wav)
        .args(["-of"])
        .arg(request.output_base)
        .args([
            "-t",
            &request.threads.to_string(),
            "-p",
            "1",
            "-fa",
            "-l",
            "auto",
            "-ojf",
            "-pp",
            "-tp",
            "0.0",
            "-tpi",
            "0.2",
            "--vad",
            "--vad-model",
        ]);
    command.arg(request.vad_model).args([
        "--vad-threshold",
        "0.50",
        "--vad-min-speech-duration-ms",
        "250",
        "--vad-min-silence-duration-ms",
        "350",
        "--vad-max-speech-duration-s",
        "240",
        "--vad-speech-pad-ms",
        "120",
        "--vad-samples-overlap",
        "0.10",
    ]);
    if !uses_metal_backend() {
        command.arg("-ng");
    }
    command
}
