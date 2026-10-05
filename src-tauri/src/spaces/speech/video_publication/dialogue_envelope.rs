use std::path::Path;

use tokio_util::sync::CancellationToken;

use super::super::video_config::SourceDialogueInterval;
use super::SAMPLE_RATE;

pub(super) fn write(
    path: &Path,
    sample_count: u64,
    intervals: &[SourceDialogueInterval],
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let mut windows = intervals
        .iter()
        .map(|window| {
            (
                window.start_ms * u64::from(SAMPLE_RATE) / 1000,
                window.end_ms * u64::from(SAMPLE_RATE) / 1000,
            )
        })
        .collect::<Vec<_>>();
    windows.sort_unstable();
    let mut merged: Vec<(u64, u64)> = Vec::new();
    for (start, end) in windows {
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
            .map_or(0.0, |(start, end)| {
                let transition = (u64::from(SAMPLE_RATE) / 100).min((end - start) / 2).max(1);
                ((index - start) as f32 / transition as f32)
                    .min((end - 1 - index) as f32 / transition as f32)
                    .clamp(0.0, 1.0)
            });
        writer
            .write_sample(value)
            .map_err(|error| error.to_string())?;
    }
    writer.finalize().map_err(|error| error.to_string())
}
