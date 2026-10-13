use super::transcription::Engine;
use crate::cancellation::Cancellation;
use knovum_inference_contracts::speech_wire::LanguagePolicy;
use std::{
    ffi::{CStr, CString},
    ops::Range,
};

pub(super) fn select(
    engine: &Engine,
    samples: &[f32],
    policy: &LanguagePolicy,
    fixed: &CStr,
    regions: &[Range<u64>],
    cancel: &Cancellation,
) -> Result<CString, String> {
    if matches!(policy, LanguagePolicy::Fixed) {
        return Ok(fixed.to_owned());
    }
    let start_ms = best_detection_start(regions);
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
    let max_id = unsafe { engine.api.whisper_lang_max_id() };
    let mut probs = if (0..=256).contains(&max_id) {
        vec![0.0f32; (max_id as usize) + 1]
    } else {
        Vec::new()
    };
    let probs_ptr = if probs.is_empty() {
        std::ptr::null_mut()
    } else {
        probs.as_mut_ptr()
    };
    let id = unsafe {
        engine.api.whisper_lang_auto_detect_with_state(
            engine.context,
            engine.state,
            0,
            i32::from(engine.threads),
            probs_ptr,
        )
    };
    cancel.check()?;
    if id < 0 {
        return Err("transcription_failed: Whisper speech language detection failed".into());
    }
    let mut selected_id = id;
    if !probs.is_empty() {
        let top_prob = probs.get(id as usize).copied().unwrap_or(0.0);
        if top_prob < 0.45 {
            if let Some(second_region) = regions.iter().find(|r| {
                r.start > start_ms.saturating_add(10_000) && r.end.saturating_sub(r.start) >= 2_000
            }) {
                if let Ok(second_speech) = window(samples, second_region.start) {
                    let mut second_probs = vec![0.0f32; probs.len()];
                    let status2 = unsafe {
                        engine.api.whisper_pcm_to_mel_with_state(
                            engine.context,
                            engine.state,
                            second_speech.as_ptr(),
                            second_speech.len() as i32,
                            i32::from(engine.threads),
                        )
                    };
                    if status2 == 0 {
                        let id2 = unsafe {
                            engine.api.whisper_lang_auto_detect_with_state(
                                engine.context,
                                engine.state,
                                0,
                                i32::from(engine.threads),
                                second_probs.as_mut_ptr(),
                            )
                        };
                        if id2 >= 0 {
                            for (p1, p2) in probs.iter_mut().zip(second_probs.iter()) {
                                *p1 += *p2;
                            }
                            if let Some((combined_id, _)) = probs
                                .iter()
                                .enumerate()
                                .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
                            {
                                selected_id = combined_id as i32;
                            }
                        }
                    }
                }
            }
        }
    }
    let language = unsafe { engine.api.whisper_lang_str(selected_id) };
    if language.is_null() {
        return Err("invalid_output: Whisper detected an unknown language".into());
    }
    Ok(unsafe { CStr::from_ptr(language) }.to_owned())
}

pub(super) fn best_detection_start(regions: &[Range<u64>]) -> u64 {
    regions
        .iter()
        .find(|region| region.end.saturating_sub(region.start) >= 2_000)
        .or_else(|| regions.iter().max_by_key(|region| region.end.saturating_sub(region.start)))
        .or_else(|| regions.first())
        .map(|region| region.start)
        .unwrap_or(0)
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
