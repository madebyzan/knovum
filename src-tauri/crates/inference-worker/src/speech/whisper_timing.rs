use super::transcription::Engine;
use knovum_inference_contracts::speech_wire::{SpeechTranscriptSegment, SpeechWordTiming};
use std::ffi::CStr;

pub(super) fn segment(
    engine: &Engine,
    index: i32,
    end: usize,
    language: &str,
) -> Result<SpeechTranscriptSegment, String> {
    let start_ms = time(
        unsafe {
            engine
                .api
                .whisper_full_get_segment_t0_from_state(engine.state, index)
        },
        end,
    )?;
    let end_ms = time(
        unsafe {
            engine
                .api
                .whisper_full_get_segment_t1_from_state(engine.state, index)
        },
        end,
    )?;
    if start_ms > end_ms {
        return Err("invalid_output: Whisper segment timeline is reversed".into());
    }
    let text = unsafe {
        engine
            .api
            .whisper_full_get_segment_text_from_state(engine.state, index)
    };
    if text.is_null() {
        return Err("invalid_output: Whisper segment text is missing".into());
    }
    let text = unsafe { CStr::from_ptr(text) }
        .to_str()
        .map_err(|error| format!("invalid_output: {error}"))?
        .to_owned();
    let mut words = Vec::<SpeechWordTiming>::new();
    let mut pending = Vec::new();
    let mut token_start = None;
    let count = unsafe {
        engine
            .api
            .whisper_full_n_tokens_from_state(engine.state, index)
    };
    if !(0..=8192).contains(&count) {
        return Err("invalid_output: Whisper token count exceeds its budget".into());
    }
    for token in 0..count {
        let bytes = unsafe {
            engine.api.whisper_full_get_token_text_from_state(
                engine.context,
                engine.state,
                index,
                token,
            )
        };
        if bytes.is_null() {
            return Err("invalid_output: Whisper token text is missing".into());
        }
        let bytes = unsafe { CStr::from_ptr(bytes) }.to_bytes();
        let data = unsafe {
            engine
                .api
                .whisper_full_get_token_data_from_state(engine.state, index, token)
        };
        if data.id >= unsafe { engine.api.whisper_token_eot(engine.context) } {
            continue;
        }
        let begin = time(data.t0, end)?;
        let finish = time(data.t1, end)?;
        if begin > finish || !data.p.is_finite() || !(0.0..=1.0).contains(&data.p) {
            return Err("invalid_output: Whisper token metadata is invalid".into());
        }
        token_start.get_or_insert(begin);
        pending.extend_from_slice(bytes);
        // A token may split a Unicode character; publish only complete UTF-8 sequences.
        if let Ok(piece) = std::str::from_utf8(&pending) {
            super::whisper_words::append(
                &mut words,
                piece,
                token_start.unwrap_or(begin),
                finish,
                data.p,
                language,
            );
            pending.clear();
            token_start = None;
        }
    }
    if !pending.is_empty() {
        return Err("invalid_output: Whisper returned incomplete Unicode token text".into());
    }
    super::whisper_words::finish(&mut words, start_ms, end_ms);
    let end_ms = if let Some(last_word) = words.last() {
        if end_ms > last_word.end_ms.saturating_add(1_000) {
            last_word.end_ms.saturating_add(200).min(end_ms)
        } else {
            end_ms
        }
    } else {
        end_ms
    };
    let start_ms = if let Some(first_word) = words.first() {
        if first_word.start_ms > start_ms.saturating_add(1_500) {
            first_word.start_ms.saturating_sub(100).max(start_ms)
        } else {
            start_ms
        }
    } else {
        start_ms
    };
    if start_ms > end_ms {
        return Err("invalid_output: Whisper segment timeline is reversed".into());
    }
    Ok(SpeechTranscriptSegment {
        start_ms,
        end_ms,
        text,
        words,
    })
}
fn time(centiseconds: i64, end: usize) -> Result<u64, String> {
    let ms = u64::try_from(centiseconds)
        .ok()
        .and_then(|value| value.checked_mul(10))
        .ok_or("invalid_output: Whisper timestamp is invalid")?;
    // Upstream pads the final inference window with 30 seconds of zeros.
    // Validate that Whisper context, then publish only the original audio timeline.
    if ms > end as u64 / 16 + 30_000 {
        return Err(format!(
            "invalid_output: Whisper timestamp {ms} exceeds its {} ms source audio",
            end as u64 / 16
        ));
    }
    Ok(ms.min(end as u64 / 16))
}

#[cfg(test)]
#[path = "tests/whisper_timing.rs"]
mod tests;
