use std::{
    fs,
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
};

use tokio_util::sync::CancellationToken;

const SAMPLE_RATE: u32 = 48_000;

pub(super) fn sample_count(path: &Path) -> Result<u64, String> {
    let reader = hound::WavReader::open(path).map_err(|error| error.to_string())?;
    let spec = reader.spec();
    if spec.sample_rate != SAMPLE_RATE
        || spec.channels != 1
        || spec.bits_per_sample != 16
        || spec.sample_format != hound::SampleFormat::Int
    {
        return Err("Video dubbing WAV is not mono PCM16 at 48 kHz".into());
    }
    Ok(u64::from(reader.duration()))
}

pub(super) fn write_header(output: &mut fs::File, data_bytes: u64) -> Result<(), String> {
    let data = u32::try_from(data_bytes).map_err(|_| "Video voice lane exceeds WAV size limits")?;
    let riff = data
        .checked_add(36)
        .ok_or("Video voice lane exceeds WAV size limits")?;
    output
        .write_all(b"RIFF")
        .and_then(|_| output.write_all(&riff.to_le_bytes()))
        .and_then(|_| output.write_all(b"WAVEfmt "))
        .and_then(|_| output.write_all(&16_u32.to_le_bytes()))
        .and_then(|_| output.write_all(&1_u16.to_le_bytes()))
        .and_then(|_| output.write_all(&1_u16.to_le_bytes()))
        .and_then(|_| output.write_all(&SAMPLE_RATE.to_le_bytes()))
        .and_then(|_| output.write_all(&(SAMPLE_RATE * 2).to_le_bytes()))
        .and_then(|_| output.write_all(&2_u16.to_le_bytes()))
        .and_then(|_| output.write_all(&16_u16.to_le_bytes()))
        .and_then(|_| output.write_all(b"data"))
        .and_then(|_| output.write_all(&data.to_le_bytes()))
        .map_err(|error| error.to_string())
}

pub(super) fn write_silence(
    output: &mut fs::File,
    samples: u64,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let mut bytes = samples
        .checked_mul(2)
        .ok_or("Video silence size overflow")?;
    let zeros = [0_u8; 1024 * 1024];
    while bytes > 0 {
        if cancellation.is_cancelled() {
            return Err("cancelled".into());
        }
        let count = usize::try_from(bytes.min(zeros.len() as u64))
            .map_err(|_| "Video silence size overflow")?;
        output
            .write_all(&zeros[..count])
            .map_err(|error| error.to_string())?;
        bytes -= count as u64;
    }
    Ok(())
}

pub(super) fn copy_samples(
    path: &Path,
    output: &mut fs::File,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let (offset, size) = data_range(path)?;
    let mut input = fs::File::open(path).map_err(|error| error.to_string())?;
    input
        .seek(SeekFrom::Start(offset))
        .map_err(|error| error.to_string())?;
    let mut input = input.take(size);
    let mut remaining = size;
    let mut buffer = [0_u8; 1024 * 1024];
    while remaining > 0 {
        if cancellation.is_cancelled() {
            return Err("cancelled".into());
        }
        let limit = usize::try_from(remaining.min(buffer.len() as u64))
            .map_err(|_| "Video speech WAV size overflow")?;
        let read = input
            .read(&mut buffer[..limit])
            .map_err(|error| error.to_string())?;
        if read == 0 {
            return Err("Video speech WAV ended before its data chunk".into());
        }
        output
            .write_all(&buffer[..read])
            .map_err(|error| error.to_string())?;
        remaining -= read as u64;
    }
    Ok(())
}

fn data_range(path: &Path) -> Result<(u64, u64), String> {
    let mut input = fs::File::open(path).map_err(|error| error.to_string())?;
    let length = input.metadata().map_err(|error| error.to_string())?.len();
    let mut header = [0_u8; 12];
    input
        .read_exact(&mut header)
        .map_err(|error| error.to_string())?;
    if &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" {
        return Err("Video speech WAV container is invalid".into());
    }
    let mut offset = 12_u64;
    while offset + 8 <= length {
        input
            .seek(SeekFrom::Start(offset))
            .map_err(|error| error.to_string())?;
        let mut chunk = [0_u8; 8];
        input
            .read_exact(&mut chunk)
            .map_err(|error| error.to_string())?;
        let size = u64::from(u32::from_le_bytes(
            chunk[4..8].try_into().map_err(|_| "Invalid WAV chunk")?,
        ));
        let start = offset + 8;
        if &chunk[0..4] == b"data" && start.checked_add(size).is_some_and(|end| end <= length) {
            return Ok((start, size));
        }
        offset = start
            .checked_add(size)
            .and_then(|end| end.checked_add(size % 2))
            .ok_or("Video speech WAV chunk offset overflow")?;
    }
    Err("Video speech WAV data chunk is missing".into())
}
