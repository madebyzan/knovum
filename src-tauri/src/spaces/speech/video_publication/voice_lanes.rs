use super::{mixing::TimedRender, SAMPLE_RATE};
use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    fs,
    path::{Path, PathBuf},
};
use tokio_util::sync::CancellationToken;

pub(super) fn assign(timed: &[TimedRender]) -> Vec<Vec<usize>> {
    let mut available = BinaryHeap::<Reverse<(u64, usize)>>::new();
    let mut lanes: Vec<Vec<usize>> = Vec::new();
    for (index, item) in timed.iter().enumerate() {
        let lane = if let Some(Reverse((end, lane))) = available.peek().copied() {
            if end <= item.start_sample {
                available.pop();
                lane
            } else {
                lanes.len()
            }
        } else {
            0
        };
        if lane == lanes.len() {
            lanes.push(Vec::new());
        }
        lanes[lane].push(index);
        available.push(Reverse((item.end_sample, lane)));
    }
    lanes
}

pub(super) fn write(
    path: &Path,
    lane: &[usize],
    timed: &[TimedRender],
    fitted: &[PathBuf],
    video_samples: u64,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let part = path.with_extension("wav.part");
    let data_bytes = video_samples
        .checked_mul(4)
        .ok_or("Video voice lane size overflow")?;
    if data_bytes > u64::from(u32::MAX) - 68 {
        return Err("Video voice lane exceeds WAV size limits".into());
    }
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&part)
        .map_err(|error| error.to_string())?;
    let spec = super::super::audio_format::WavPrecision::Float32.spec(SAMPLE_RATE, 1);
    let mut writer = hound::WavWriter::new(std::io::BufWriter::new(&mut output), spec)
        .map_err(|error| error.to_string())?;
    let mut cursor = 0_u64;
    for index in lane {
        crate::media::wav::write_silence(
            &mut writer,
            timed[*index].start_sample - cursor,
            cancellation,
        )?;
        crate::media::wav::copy_float_samples(&fitted[*index], &mut writer, cancellation)?;
        cursor = timed[*index].end_sample;
    }
    crate::media::wav::write_silence(&mut writer, video_samples - cursor, cancellation)?;
    writer.finalize().map_err(|error| error.to_string())?;
    output.sync_all().map_err(|error| error.to_string())?;
    fs::rename(&part, path).map_err(|error| error.to_string())
}
