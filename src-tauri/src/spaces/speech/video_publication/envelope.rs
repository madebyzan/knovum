//! Gain envelopes over speech windows, written as mono float WAVs that the mix graph multiplies
//! into a bus. Two shapes ship: the swap envelope that crossfades between the separated bed and
//! the preserved original track, and the duck envelope that dips the bed under synthesized
//! dialogue.

use std::path::Path;

use tokio_util::sync::CancellationToken;

use super::super::video_config::SourceDialogueInterval;
use super::{fitting::Placement, SAMPLE_RATE};

/// The swap envelope's crossfade: long enough to avoid a click, short enough to be inaudible.
const SWAP_RAMP_SAMPLES: u64 = SAMPLE_RATE as u64 / 100;

pub(super) struct EnvelopeShape {
    /// Value held inside the speech windows.
    pub inside: f32,
    /// Value held outside the speech windows.
    pub outside: f32,
    /// How far before a window the envelope starts leaving `outside`.
    pub lookahead_samples: u64,
    /// How long the envelope takes to reach `inside` once the window has started.
    pub attack_samples: u64,
    /// How long the envelope takes to leave `inside` before the window ends.
    pub tail_samples: u64,
    /// How far after a window the envelope finishes returning to `outside`.
    pub release_samples: u64,
}

/// The original-preserving swap envelope: the separated bed inside source-dialogue and dubbing
/// windows, the preserved original outside, with short linear crossfades at the boundaries.
pub(super) fn write_swap(
    path: &Path,
    sample_count: u64,
    intervals: &[SourceDialogueInterval],
    placements: &[Placement],
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let windows = intervals
        .iter()
        .map(|window| {
            (
                window.start_ms * u64::from(SAMPLE_RATE) / 1000,
                window.end_ms * u64::from(SAMPLE_RATE) / 1000,
            )
        })
        .chain(placement_windows(sample_count, placements));
    write(
        path,
        sample_count,
        windows,
        &EnvelopeShape {
            inside: 1.0,
            outside: 0.0,
            lookahead_samples: 0,
            attack_samples: SWAP_RAMP_SAMPLES,
            tail_samples: SWAP_RAMP_SAMPLES,
            release_samples: 0,
        },
        cancellation,
    )
}

/// The ducking envelope: the bed plays at full level except under synthesized dialogue, where it
/// sits at [`super::mastering::duck_level`], already down when the first word starts and
/// recovering slowly enough that the dip is never perceived as pumping.
pub(super) fn write_duck(
    path: &Path,
    sample_count: u64,
    placements: &[Placement],
    cancellation: &CancellationToken,
) -> Result<(), String> {
    write(
        path,
        sample_count,
        placement_windows(sample_count, placements),
        &EnvelopeShape {
            inside: super::mastering::duck_level(),
            outside: 1.0,
            lookahead_samples: super::mastering::DUCK_ATTACK_SAMPLES,
            attack_samples: 0,
            tail_samples: 0,
            release_samples: super::mastering::DUCK_RELEASE_SAMPLES,
        },
        cancellation,
    )
}

fn placement_windows(
    sample_count: u64,
    placements: &[Placement],
) -> impl Iterator<Item = (u64, u64)> + '_ {
    placements.iter().map(move |placement| {
        (
            placement.start_sample,
            placement
                .start_sample
                .saturating_add(placement.sample_count)
                .min(sample_count),
        )
    })
}

fn write(
    path: &Path,
    sample_count: u64,
    windows: impl Iterator<Item = (u64, u64)>,
    shape: &EnvelopeShape,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let mut expanded = windows
        .map(|(start, end)| {
            (
                start.saturating_sub(shape.lookahead_samples),
                end.saturating_add(shape.release_samples).min(sample_count),
            )
        })
        .collect::<Vec<_>>();
    expanded.sort_unstable();
    let mut merged: Vec<(u64, u64)> = Vec::new();
    for (start, end) in expanded {
        if let Some(previous) = merged.last_mut().filter(|previous| start <= previous.1) {
            previous.1 = previous.1.max(end);
        } else {
            merged.push((start, end));
        }
    }
    let mut writer = hound::WavWriter::create(
        path,
        hound::WavSpec {
            channels: 1,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .map_err(|error| error.to_string())?;
    let mut window_index = 0;
    for index in 0..sample_count {
        if index % 16_384 == 0 && cancellation.is_cancelled() {
            return Err("cancelled".into());
        }
        while merged
            .get(window_index)
            .is_some_and(|(_, end)| index >= *end)
        {
            window_index += 1;
        }
        let value = merged
            .get(window_index)
            .filter(|(start, end)| index >= *start && index < *end)
            .map_or(shape.outside, |(start, end)| {
                // Ramps never cross: on windows shorter than both ramps the envelope peaks at
                // the midpoint instead of clipping through `inside`.
                let length = end - start;
                let rise = (shape.lookahead_samples + shape.attack_samples)
                    .min(length / 2)
                    .max(1);
                let fall = (shape.tail_samples + shape.release_samples)
                    .min(length / 2)
                    .max(1);
                let progress = ((index - start) as f32 / rise as f32)
                    .min((end - 1 - index) as f32 / fall as f32)
                    .clamp(0.0, 1.0);
                shape.outside + (shape.inside - shape.outside) * progress
            });
        writer
            .write_sample(value)
            .map_err(|error| error.to_string())?;
    }
    writer.finalize().map_err(|error| error.to_string())
}

#[cfg(test)]
#[path = "tests/envelope.rs"]
mod tests;
