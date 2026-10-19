//! A synthesized line is the speech itself, not the silence a model happens to leave around it.
//!
//! Models pad their output by varying amounts, and that padding is indistinguishable from spoken
//! time to everything downstream: a dubbed line would start late and hold the floor after it had
//! stopped speaking, and an audiobook's configured paragraph pause would be that pause plus
//! whatever the model added. Trimming here, where a line's audio is assembled, is what makes the
//! stored duration of a line mean the duration of its speech.

use std::ops::Range;

/// Speech is looked for in short frames, so a boundary is found close to the word itself.
const FRAME_MS: u64 = 10;
/// A frame whose loudest sample stays under -45 dBFS carries no speech worth keeping.
const SILENCE_PEAK: f32 = 0.005_623_413;
/// Kept on both sides of the speech, so no onset or tail is ever clipped by the trim.
const GUARD_MS: u64 = 20;

/// The span of `pcm` that holds speech, as a byte range into mono 32-bit float samples.
pub(super) fn speech_span(pcm: &[u8], sample_rate: u32) -> Result<Range<usize>, String> {
    let frame = frame_bytes(sample_rate, FRAME_MS)?;
    let speaking = |window: &&[u8]| {
        window
            .chunks_exact(4)
            .any(|sample| sample_peak(sample) >= SILENCE_PEAK)
    };
    let frames = pcm.chunks(frame).collect::<Vec<_>>();
    let first = frames
        .iter()
        .position(speaking)
        .ok_or("invalid_output: synthesis returned silence")?;
    let last = frames
        .iter()
        .rposition(speaking)
        .ok_or("invalid_output: synthesis returned silence")?;
    let guard = frame_bytes(sample_rate, GUARD_MS)?;
    let start = (first * frame).saturating_sub(guard);
    let end = ((last + 1) * frame + guard).min(pcm.len());
    Ok(start..end)
}

fn frame_bytes(sample_rate: u32, duration_ms: u64) -> Result<usize, String> {
    let samples = u64::from(sample_rate)
        .checked_mul(duration_ms)
        .ok_or("invalid_output: synthesis sample rate is invalid")?
        / 1_000;
    usize::try_from(samples.max(1) * 4)
        .map_err(|_| "invalid_output: synthesis sample rate is invalid".into())
}

fn sample_peak(sample: &[u8]) -> f32 {
    f32::from_le_bytes([sample[0], sample[1], sample[2], sample[3]]).abs()
}

#[cfg(test)]
#[path = "tests/trim.rs"]
mod tests;
