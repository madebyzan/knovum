use crate::{artifact, cancellation::Cancellation};
use knovum_inference_contracts::speech_wire::{AudioFileReference, SynthesizedAudioFileReference};
use sha2::{Digest, Sha256};
use std::{
    io::{Cursor, Read},
    path::Path,
};

pub(crate) struct Audio {
    pub(crate) samples: Vec<f32>,
    pub(crate) channels: u16,
}
impl Audio {
    pub(crate) fn frames(&self) -> usize {
        self.samples.len() / usize::from(self.channels)
    }
}

pub(crate) fn read(
    reference: &AudioFileReference,
    rate: u32,
    channels: u16,
    maximum_frames: usize,
    cancel: &Cancellation,
) -> Result<Audio, String> {
    cancel.check()?;
    let maximum = (maximum_frames as u64)
        .checked_mul(u64::from(channels) * 2)
        .and_then(|size| size.checked_add(65_536))
        .ok_or("invalid_input: audio byte budget overflows")?;
    if reference.media_type != "audio/wav"
        || reference.sample_format != "pcm_s16le"
        || reference.sample_rate != rate
        || reference.channels != channels
        || channels == 0
        || reference.byte_length > maximum
        || reference.byte_length < 44
        || !Path::new(&reference.path).is_absolute()
    {
        return Err("invalid_input: audio reference differs from its admitted format".into());
    }
    let file = artifact::open_regular(Path::new(&reference.path))?;
    if file.metadata().map_err(|error| error.to_string())?.len() != reference.byte_length {
        return Err("invalid_input: audio size differs from its reference".into());
    }
    let mut bytes = Vec::with_capacity(reference.byte_length as usize);
    file.take(reference.byte_length + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 != reference.byte_length
        || hex::encode(Sha256::digest(&bytes)) != reference.sha256
    {
        return Err("invalid_input: audio digest differs from its reference".into());
    }
    let mut reader = hound::WavReader::new(Cursor::new(bytes))
        .map_err(|error| format!("invalid_input: {error}"))?;
    let spec = reader.spec();
    if spec.sample_rate != rate
        || spec.channels != channels
        || spec.bits_per_sample != 16
        || spec.sample_format != hound::SampleFormat::Int
        || reader.len() == 0
        || !(reader.len() as usize).is_multiple_of(usize::from(channels))
        || reader.duration() as usize > maximum_frames
    {
        return Err("invalid_input: WAV header exceeds its admitted audio timeline".into());
    }
    let mut samples = Vec::with_capacity(reader.len() as usize);
    for (index, sample) in reader.samples::<i16>().enumerate() {
        if index % 16_384 == 0 {
            cancel.check()?;
        }
        samples
            .push(f32::from(sample.map_err(|error| format!("invalid_input: {error}"))?) / 32_768.0);
    }
    cancel.check()?;
    Ok(Audio { samples, channels })
}

pub(crate) fn pcm16(value: f32) -> i16 {
    (value * 32_768.0)
        .round_ties_even()
        .clamp(-32_768.0, 32_767.0) as i16
}

pub(crate) fn write_wave(
    directory: &Path,
    name: &str,
    rate: u32,
    channels: u16,
    samples: &[f32],
) -> Result<SynthesizedAudioFileReference, String> {
    if channels == 0
        || rate == 0
        || samples.is_empty()
        || !samples.len().is_multiple_of(usize::from(channels))
        || samples.len() > 128 * 1024 * 1024
        || samples.iter().any(|value| !value.is_finite())
    {
        return Err("invalid_output: audio samples exceed their output contract".into());
    }
    let mut bytes = Cursor::new(Vec::with_capacity(samples.len() * 2 + 44));
    let spec = hound::WavSpec {
        channels,
        sample_rate: rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::new(&mut bytes, spec).map_err(|error| error.to_string())?;
    for &sample in samples {
        writer
            .write_sample(pcm16(sample))
            .map_err(|error| error.to_string())?;
    }
    writer.finalize().map_err(|error| error.to_string())?;
    let bytes = bytes.into_inner();
    artifact::write_bytes(directory, name, &bytes)?;
    Ok(SynthesizedAudioFileReference {
        file_name: name.into(),
        media_type: "audio/wav".into(),
        byte_length: bytes.len() as u64,
        sha256: hex::encode(Sha256::digest(&bytes)),
        sample_rate: rate,
        channels,
        sample_format: "pcm_s16le".into(),
    })
}
