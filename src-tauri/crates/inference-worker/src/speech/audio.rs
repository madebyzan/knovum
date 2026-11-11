use crate::{artifact, cancellation::Cancellation};
use knovum_inference_contracts::speech_wire::{AudioFileReference, SynthesizedAudioFileReference};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{BufReader, BufWriter, Cursor, Read},
    path::{Path, PathBuf},
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
    let mut stream = WaveStream::open(reference, rate, channels, maximum_frames)?;
    let mut samples = Vec::with_capacity(stream.frames() * usize::from(channels));
    while stream.read(16_384, &mut samples)? > 0 {
        cancel.check()?;
    }
    stream.finish()?;
    cancel.check()?;
    Ok(Audio { samples, channels })
}

/// A 16-bit PCM WAV reference read block by block. Its bytes are hashed as they are read and the
/// digest is settled by [`WaveStream::finish`], so a source of any admitted length is verified in
/// the same single pass that reads it, without being held in memory.
pub(crate) struct WaveStream {
    reader: hound::WavReader<Hashing<BufReader<File>>>,
    channels: u16,
    frames: usize,
    sha256: String,
}

impl WaveStream {
    pub(crate) fn open(
        reference: &AudioFileReference,
        rate: u32,
        channels: u16,
        maximum_frames: usize,
    ) -> Result<Self, String> {
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
        let reader = hound::WavReader::new(Hashing {
            inner: BufReader::new(file),
            digest: Sha256::new(),
        })
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
        Ok(Self {
            frames: reader.duration() as usize,
            reader,
            channels,
            sha256: reference.sha256.clone(),
        })
    }

    pub(crate) fn frames(&self) -> usize {
        self.frames
    }

    /// Appends up to `frames` interleaved frames to `into` and returns how many were read; zero
    /// once the stream is exhausted.
    pub(crate) fn read(&mut self, frames: usize, into: &mut Vec<f32>) -> Result<usize, String> {
        let before = into.len();
        for sample in self
            .reader
            .samples::<i16>()
            .take(frames * usize::from(self.channels))
        {
            into.push(
                f32::from(sample.map_err(|error| format!("invalid_input: {error}"))?) / 32_768.0,
            );
        }
        Ok((into.len() - before) / usize::from(self.channels))
    }

    /// Reads whatever follows the samples and settles the digest of the whole file.
    pub(crate) fn finish(self) -> Result<(), String> {
        let mut rest = self.reader.into_inner();
        std::io::copy(&mut rest, &mut std::io::sink()).map_err(|error| error.to_string())?;
        if hex::encode(rest.digest.finalize()) != self.sha256 {
            return Err("invalid_input: audio digest differs from its reference".into());
        }
        Ok(())
    }
}

/// A reader that hashes every byte it hands out.
struct Hashing<R> {
    inner: R,
    digest: Sha256,
}

impl<R: Read> Read for Hashing<R> {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let read = self.inner.read(buffer)?;
        self.digest.update(&buffer[..read]);
        Ok(read)
    }
}

/// A 16-bit PCM WAV artifact written frame block by frame block.
pub(crate) struct WaveWriter {
    writer: hound::WavWriter<BufWriter<File>>,
    directory: PathBuf,
    name: String,
    rate: u32,
    channels: u16,
}

impl WaveWriter {
    pub(crate) fn create(
        directory: &Path,
        name: &str,
        rate: u32,
        channels: u16,
    ) -> Result<Self, String> {
        if !artifact::valid_name(name, 160, true) || rate == 0 || channels == 0 {
            return Err("invalid_output: audio artifact is invalid".into());
        }
        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join(name))
            .map_err(|error| format!("resource_exhausted: {error}"))?;
        let spec = hound::WavSpec {
            channels,
            sample_rate: rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        Ok(Self {
            writer: hound::WavWriter::new(BufWriter::new(file), spec)
                .map_err(|error| error.to_string())?,
            directory: directory.to_path_buf(),
            name: name.into(),
            rate,
            channels,
        })
    }

    pub(crate) fn write(&mut self, samples: &[f32]) -> Result<(), String> {
        if !samples.len().is_multiple_of(usize::from(self.channels))
            || samples.iter().any(|value| !value.is_finite())
        {
            return Err("invalid_output: audio samples exceed their output contract".into());
        }
        for &sample in samples {
            self.writer
                .write_sample(pcm16(sample))
                .map_err(|error| format!("resource_exhausted: {error}"))?;
        }
        Ok(())
    }

    /// Completes the file and describes it as the artifact `prefix/name`.
    pub(crate) fn finish(self, prefix: &str) -> Result<SynthesizedAudioFileReference, String> {
        self.writer
            .finalize()
            .map_err(|error| format!("resource_exhausted: {error}"))?;
        let mut file = artifact::open_regular(&self.directory.join(&self.name))?;
        let mut digest = Sha256::new();
        let mut byte_length = 0_u64;
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
            if read == 0 {
                break;
            }
            digest.update(&buffer[..read]);
            byte_length += read as u64;
        }
        Ok(SynthesizedAudioFileReference {
            file_name: format!("{prefix}/{}", self.name),
            media_type: "audio/wav".into(),
            byte_length,
            sha256: hex::encode(digest.finalize()),
            sample_rate: self.rate,
            channels: self.channels,
            sample_format: "pcm_s16le".into(),
        })
    }
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
