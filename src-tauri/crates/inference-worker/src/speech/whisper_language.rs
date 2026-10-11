use super::transcription::Engine;
use crate::cancellation::Cancellation;
use knovum_inference_contracts::speech_wire::LanguagePolicy;
use std::ffi::{CStr, CString};

pub(super) fn select(
    engine: &Engine,
    samples: &[f32],
    policy: &LanguagePolicy,
    fixed: &CStr,
    start_ms: u64,
    cancel: &Cancellation,
) -> Result<CString, String> {
    if matches!(policy, LanguagePolicy::Fixed) {
        return Ok(fixed.to_owned());
    }
    // Upstream full() detects at offset zero. Detect from the first speech window
    // through its public API, then decode the complete original waveform.
    let speech = window(samples, start_ms)?;
    cancel.check()?;
    let status = unsafe {
        engine.api.whisper_pcm_to_mel_with_state(
            engine.context,
            engine.state,
            speech.as_ptr(),
            speech.len() as i32,
            i32::from(engine.threads),
        )
    };
    cancel.check()?;
    if status != 0 {
        return Err(format!(
            "transcription_failed: language spectrogram status {status}"
        ));
    }
    let id = unsafe {
        engine.api.whisper_lang_auto_detect_with_state(
            engine.context,
            engine.state,
            0,
            i32::from(engine.threads),
            std::ptr::null_mut(),
        )
    };
    cancel.check()?;
    if id < 0 {
        return Err("transcription_failed: Whisper speech language detection failed".into());
    }
    let language = unsafe { engine.api.whisper_lang_str(id) };
    if language.is_null() {
        return Err("invalid_output: Whisper detected an unknown language".into());
    }
    Ok(unsafe { CStr::from_ptr(language) }.to_owned())
}

fn window(samples: &[f32], start_ms: u64) -> Result<&[f32], String> {
    let start = usize::try_from(start_ms)
        .ok()
        .and_then(|value| value.checked_mul(16))
        .filter(|value| *value < samples.len())
        .ok_or("invalid_output: speech language window exceeds its source audio")?;
    Ok(&samples[start..start.saturating_add(16_000 * 30).min(samples.len())])
}

#[cfg(test)]
#[path = "tests/whisper_language.rs"]
mod tests;
