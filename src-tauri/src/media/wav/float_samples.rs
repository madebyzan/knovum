use std::{
    io::{Seek, Write},
    path::Path,
};
use tokio_util::sync::CancellationToken;

const CANCELLATION_INTERVAL: u64 = 16_384;

pub(crate) fn write_silence<W: Write + Seek>(
    writer: &mut hound::WavWriter<W>,
    sample_count: u64,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    for index in 0..sample_count {
        if index.is_multiple_of(CANCELLATION_INTERVAL) && cancellation.is_cancelled() {
            return Err("cancelled".into());
        }
        writer
            .write_sample(0.0_f32)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub(crate) fn copy_float_samples<W: Write + Seek>(
    path: &Path,
    writer: &mut hound::WavWriter<W>,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let metadata = super::metadata(path)?;
    if metadata.spec != writer.spec() || metadata.spec.sample_format != hound::SampleFormat::Float {
        return Err("WAV source differs from the output float format".into());
    }
    let mut reader = hound::WavReader::open(path).map_err(|error| error.to_string())?;
    for (index, sample) in reader.samples::<f32>().enumerate() {
        if (index as u64).is_multiple_of(CANCELLATION_INTERVAL) && cancellation.is_cancelled() {
            return Err("cancelled".into());
        }
        writer
            .write_sample(sample.map_err(|error| error.to_string())?)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}
