use super::{fitting::Placement, SAMPLE_RATE};
use std::{
    fs,
    path::{Path, PathBuf},
};
use tokio_util::sync::CancellationToken;

/// Writes the dubbed dialogue as one track the length of the picture: every line at the time it
/// was placed, silence everywhere else. Placements never overlap, so one track holds them all.
pub(super) fn write(
    path: &Path,
    placements: &[Placement],
    fitted: &[PathBuf],
    video_samples: u64,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let part = path.with_extension("wav.part");
    let data_bytes = video_samples
        .checked_mul(4)
        .ok_or("Video dialogue track size overflow")?;
    if data_bytes > u64::from(u32::MAX) - 68 {
        return Err("Video dialogue track exceeds WAV size limits".into());
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
    for (placement, line) in placements.iter().zip(fitted) {
        crate::media::wav::write_silence(
            &mut writer,
            placement.start_sample - cursor,
            cancellation,
        )?;
        crate::media::wav::copy_float_samples(line, &mut writer, cancellation)?;
        cursor = placement.end_sample();
    }
    crate::media::wav::write_silence(&mut writer, video_samples - cursor, cancellation)?;
    writer.finalize().map_err(|error| error.to_string())?;
    output.sync_all().map_err(|error| error.to_string())?;
    fs::rename(&part, path).map_err(|error| error.to_string())
}
