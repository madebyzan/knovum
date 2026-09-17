//! Parses whisper.cpp JSON output into neutral transcript data.

use std::{fs, path::Path};

use serde::Deserialize;

use super::{WhisperOutput, WhisperSegment};
use crate::media::subtitle_timing::{self, TimedToken};

#[derive(Deserialize)]
struct CliOutput {
    result: CliLanguage,
    transcription: Vec<CliSegment>,
}

#[derive(Deserialize)]
struct CliLanguage {
    language: String,
}

#[derive(Deserialize)]
struct CliSegment {
    offsets: CliOffsets,
    text: String,
    tokens: Vec<CliToken>,
}

#[derive(Deserialize)]
struct CliToken {
    text: String,
    offsets: Option<CliOffsets>,
}

#[derive(Deserialize)]
struct CliOffsets {
    from: u64,
    to: u64,
}

pub fn read(path: &Path) -> Result<WhisperOutput, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read whisper.cpp output: {error}"))?;
    let output: CliOutput = serde_json::from_slice(&bytes)
        .map_err(|error| format!("whisper.cpp output was invalid: {error}"))?;
    let subtitle_cues = subtitle_timing::cues(
        output
            .transcription
            .iter()
            .flat_map(|segment| segment.tokens.iter())
            .filter_map(timed_token),
    );
    Ok(WhisperOutput {
        language: output.result.language,
        segments: output
            .transcription
            .into_iter()
            .map(|segment| WhisperSegment {
                start: segment.offsets.from as f64 / 1_000.0,
                end: segment.offsets.to as f64 / 1_000.0,
                text: segment.text,
            })
            .collect(),
        subtitle_cues,
    })
}

fn timed_token(token: &CliToken) -> Option<TimedToken> {
    let offsets = token.offsets.as_ref()?;
    Some(TimedToken {
        start_ms: offsets.from,
        end_ms: offsets.to,
        text: token.text.clone(),
    })
}
