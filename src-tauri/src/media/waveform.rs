use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use tokio_util::sync::CancellationToken;

const TARGET_BINS: usize = 600;

pub(crate) struct AudioWaveform {
    pub values: Vec<f32>,
    pub sample_count: u64,
}

pub fn read_waveform(
    path: &Path,
    cancellation: &CancellationToken,
) -> Result<AudioWaveform, String> {
    let file =
        File::open(path).map_err(|error| format!("failed to open media waveform: {error}"))?;
    let length = file
        .metadata()
        .map_err(|error| format!("failed to inspect media waveform: {error}"))?
        .len();
    if length % 4 != 0 {
        return Err("Media waveform contains incomplete PCM samples".into());
    }
    let samples = length / 4;
    let count = samples.min(TARGET_BINS as u64);
    let mut reader = BufReader::new(file);
    let mut values = Vec::with_capacity(count as usize);
    for index in 0..count {
        if cancellation.is_cancelled() {
            return Err(super::TRANSCRIPTION_CANCELLED.into());
        }
        let start = index * samples / count;
        let end = (index + 1) * samples / count;
        let mut peak = 0.0_f32;
        for _ in start..end {
            let mut bytes = [0; 4];
            reader
                .read_exact(&mut bytes)
                .map_err(|error| format!("failed to read media waveform: {error}"))?;
            let sample = f32::from_le_bytes(bytes);
            if !sample.is_finite() {
                return Err("Media waveform contains non-finite PCM samples".into());
            }
            peak = peak.max(sample.abs().min(1.0));
        }
        values.push(peak);
    }
    Ok(AudioWaveform {
        values,
        sample_count: samples,
    })
}
