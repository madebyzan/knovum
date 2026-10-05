mod float_samples;
pub(crate) use float_samples::{copy_float_samples, write_silence};
#[cfg(test)]
mod tests;

use std::{
    fs::File,
    io::{BufReader, Read, Seek, SeekFrom},
    path::Path,
};

pub(crate) struct WavMetadata {
    pub spec: hound::WavSpec,
    pub sample_count: u64,
    pub byte_length: u64,
}

pub(crate) fn metadata(path: &Path) -> Result<WavMetadata, String> {
    inspect(BufReader::new(
        File::open(path).map_err(|error| error.to_string())?,
    ))
}

pub(crate) fn inspect<R: Read + Seek>(mut source: R) -> Result<WavMetadata, String> {
    let byte_length = source
        .seek(SeekFrom::End(0))
        .map_err(|error| error.to_string())?;
    source
        .seek(SeekFrom::Start(0))
        .map_err(|error| error.to_string())?;
    let mut header = [0_u8; 12];
    source
        .read_exact(&mut header)
        .map_err(|error| error.to_string())?;
    // Hound decodes format/chunk metadata; this boundary also verifies physical length.
    let declared = u64::from(u32::from_le_bytes(
        header[4..8].try_into().map_err(|_| "Invalid WAV size")?,
    )) + 8;
    if declared != byte_length {
        return Err("WAV container length differs from its declared size".into());
    }
    source
        .seek(SeekFrom::Start(0))
        .map_err(|error| error.to_string())?;
    let reader = hound::WavReader::new(source).map_err(|error| error.to_string())?;
    let spec = reader.spec();
    let sample_count = u64::from(reader.duration());
    let mut source = reader.into_inner();
    let offset = source
        .stream_position()
        .map_err(|error| error.to_string())?;
    // Hound positions the stream at the data payload; its length field precedes it.
    source
        .seek(SeekFrom::Start(offset - 4))
        .map_err(|error| error.to_string())?;
    let mut data_size = [0_u8; 4];
    source
        .read_exact(&mut data_size)
        .map_err(|error| error.to_string())?;
    let data_bytes = u64::from(u32::from_le_bytes(data_size));
    if offset
        .checked_add(data_bytes)
        .is_none_or(|end| end > byte_length)
    {
        return Err("WAV data extends beyond its container".into());
    }
    Ok(WavMetadata {
        spec,
        sample_count,
        byte_length,
    })
}
