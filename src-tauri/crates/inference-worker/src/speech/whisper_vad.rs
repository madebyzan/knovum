use super::native::whisper::{whisper_vad_context, whisper_vad_segments, WhisperApi};
use crate::cancellation::Cancellation;
use std::{ffi::CStr, ops::Range};

struct Detector<'a> {
    api: &'a WhisperApi,
    context: *mut whisper_vad_context,
    segments: *mut whisper_vad_segments,
}
impl Drop for Detector<'_> {
    fn drop(&mut self) {
        unsafe {
            self.api.whisper_vad_free_segments(self.segments);
            self.api.whisper_vad_free(self.context);
        }
    }
}

pub(super) fn regions(
    api: &WhisperApi,
    model: &CStr,
    samples: &[f32],
    threads: u8,
    cancel: &Cancellation,
) -> Result<Vec<Range<u64>>, String> {
    cancel.check()?;
    let mut options = unsafe { api.whisper_vad_default_context_params() };
    options.n_threads = i32::from(threads);
    // Silero is the small CPU preprocessing stage; the ASR model keeps its admitted device.
    options.use_gpu = false;
    let context = unsafe { api.whisper_vad_init_from_file_with_params(model.as_ptr(), options) };
    if context.is_null() {
        return Err("model_corrupt: Silero VAD could not load".into());
    }
    let mut detector = Detector {
        api,
        context,
        segments: std::ptr::null_mut(),
    };
    cancel.check()?;
    detector.segments = unsafe {
        api.whisper_vad_segments_from_samples(
            context,
            api.whisper_vad_default_params(),
            samples.as_ptr(),
            samples.len() as i32,
        )
    };
    cancel.check()?;
    if detector.segments.is_null() {
        return Err("transcription_failed: Silero VAD detection failed".into());
    }
    let count = unsafe { api.whisper_vad_segments_n_segments(detector.segments) };
    if !(0..=100_000).contains(&count) {
        return Err("invalid_output: Silero VAD segment count exceeds its budget".into());
    }
    let duration_ms = samples.len() as u64 / 16;
    let mut result = Vec::with_capacity(count as usize);
    for index in 0..count {
        let start = unsafe { api.whisper_vad_segments_get_segment_t0(detector.segments, index) };
        let end = unsafe { api.whisper_vad_segments_get_segment_t1(detector.segments, index) };
        let interval = checked_region(start, end, duration_ms)?;
        if result
            .last()
            .is_some_and(|last: &Range<u64>| last.end > interval.start)
        {
            return Err("invalid_output: Silero VAD regions overlap or are unordered".into());
        }
        result.push(interval);
    }
    Ok(result)
}

fn checked_region(start: f32, end: f32, duration_ms: u64) -> Result<Range<u64>, String> {
    // The public API returns centiseconds and pads its final 512-sample window (32 ms).
    if !start.is_finite()
        || !end.is_finite()
        || start < 0.0
        || end <= start
        || f64::from(end) * 10.0 > (duration_ms + 32) as f64
    {
        return Err("invalid_output: Silero VAD timeline is invalid".into());
    }
    let start_ms = (f64::from(start) * 10.0).round() as u64;
    let end_ms = ((f64::from(end) * 10.0).round() as u64).min(duration_ms);
    if start_ms >= end_ms {
        return Err("invalid_output: Silero VAD region is outside its source audio".into());
    }
    Ok(start_ms..end_ms)
}

#[cfg(test)]
#[path = "tests/whisper_vad.rs"]
mod tests;
