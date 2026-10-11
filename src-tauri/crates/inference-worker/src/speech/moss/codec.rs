use super::super::{audio, stream::Publisher};
use super::metadata::{self, Streaming};
use crate::{
    cancellation::Cancellation,
    onnx::{self, Values},
};
use audioadapter_buffers::owned::InterleavedOwned;
use knovum_inference_contracts::speech_wire::AudioFileReference;
use ort::session::Session;
use rubato::{Fft, FixedSync, Resampler};

pub(super) struct Codec {
    pub(super) encoder: Session,
    pub(super) decoder: Session,
    metadata: Streaming,
    state: Values,
}
impl Codec {
    pub(super) fn new(
        encoder: Session,
        decoder: Session,
        metadata: Streaming,
    ) -> Result<Self, String> {
        if metadata.transformer_offsets.len() != 4 || metadata.attention_caches.len() != 12 {
            return Err("model_corrupt: codec cache inventory differs".into());
        }
        let mut codec = Self {
            encoder,
            decoder,
            metadata,
            state: Values::new(),
        };
        codec.reset()?;
        Ok(codec)
    }
    pub(super) fn reset(&mut self) -> Result<(), String> {
        self.state.clear();
        for offset in &self.metadata.transformer_offsets {
            self.state.insert(
                offset.input_name.clone(),
                onnx::tensor(
                    offset.shape.clone(),
                    vec![0i32; metadata::count(&offset.shape)?],
                )?,
            );
        }
        for cache in &self.metadata.attention_caches {
            self.state.insert(
                cache.offset_input_name.clone(),
                onnx::tensor(
                    cache.offset_shape.clone(),
                    vec![0i32; metadata::count(&cache.offset_shape)?],
                )?,
            );
            for name in [
                &cache.cached_keys_input_name,
                &cache.cached_values_input_name,
            ] {
                self.state.insert(
                    name.clone(),
                    onnx::tensor(
                        cache.cache_shape.clone(),
                        vec![0.0f32; metadata::count(&cache.cache_shape)?],
                    )?,
                );
            }
            self.state.insert(
                cache.cached_positions_input_name.clone(),
                onnx::tensor(
                    cache.positions_shape.clone(),
                    vec![-1i32; metadata::count(&cache.positions_shape)?],
                )?,
            );
        }
        Ok(())
    }
    pub(super) fn reference(
        &mut self,
        reference: &AudioFileReference,
        cancel: &Cancellation,
    ) -> Result<Vec<Vec<i32>>, String> {
        let input = audio::read(reference, 16_000, 1, 30 * 16_000, cancel)?;
        if input.frames() < 5 * 16_000 {
            return Err("invalid_input: reference audio must last five to thirty seconds".into());
        }
        let frames = input.frames();
        let source = InterleavedOwned::new_from(input.samples, 1, frames)
            .map_err(|error| format!("invalid_input: {error}"))?;
        let mut resampler = Fft::<f32>::new(16_000, 48_000, 1024, 1, FixedSync::Both)
            .map_err(|error| format!("resource_exhausted: {error}"))?;
        let mut samples = resampler
            .process_all(&source, frames, None)
            .map_err(|error| format!("inference_failed: {error}"))?
            .take_data();
        if samples.len() < frames * 3 {
            return Err("invalid_output: reference resampler omitted source tail".into());
        }
        samples.truncate(frames * 3);
        cancel.check()?;
        let frames = samples.len();
        let mut stereo = Vec::with_capacity(frames * 2);
        stereo.extend_from_slice(&samples);
        stereo.extend_from_slice(&samples);
        let feeds = Values::from([
            (
                "waveform".into(),
                onnx::tensor(vec![1, 2, frames as i64], stereo)?,
            ),
            (
                "input_lengths".into(),
                onnx::tensor(vec![1], vec![frames as i32])?,
            ),
        ]);
        let mut outputs = onnx::run(&mut self.encoder, feeds, cancel)?;
        let codes = onnx::take(&mut outputs, "audio_codes")?;
        let lengths = onnx::take(&mut outputs, "audio_code_lengths")?;
        let (_, length) = lengths
            .try_extract_tensor::<i32>()
            .map_err(|error| format!("invalid_output: {error}"))?;
        let (shape, codes) = codes
            .try_extract_tensor::<i32>()
            .map_err(|error| format!("invalid_output: {error}"))?;
        if length.len() != 1
            || length[0] <= 0
            || shape.len() != 3
            || shape[0] != 1
            || shape[2] != 16
            || i64::from(length[0]) > shape[1]
        {
            return Err("invalid_output: reference codec returned invalid codes".into());
        }
        Ok(codes[..length[0] as usize * 16]
            .chunks_exact(16)
            .map(<[i32]>::to_vec)
            .collect())
    }
    pub(super) fn publish(
        &mut self,
        frames: &[Vec<i32>],
        publisher: &mut Publisher,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        if frames.is_empty() || frames.len() > 8 || frames.iter().any(|frame| frame.len() != 16) {
            return Err("invalid_input: codec frame batch differs".into());
        }
        let mut feeds = std::mem::take(&mut self.state);
        feeds.insert(
            "audio_codes".into(),
            onnx::tensor(
                vec![1, frames.len() as i64, 16],
                frames.iter().flatten().copied().collect::<Vec<_>>(),
            )?,
        );
        feeds.insert(
            "audio_code_lengths".into(),
            onnx::tensor(vec![1], vec![frames.len() as i32])?,
        );
        let mut outputs = onnx::run(&mut self.decoder, feeds, cancel)?;
        for offset in &self.metadata.transformer_offsets {
            self.state.insert(
                offset.input_name.clone(),
                onnx::take(&mut outputs, &offset.output_name)?,
            );
        }
        for cache in &self.metadata.attention_caches {
            for (input, output) in [
                (&cache.offset_input_name, &cache.offset_output_name),
                (
                    &cache.cached_keys_input_name,
                    &cache.cached_keys_output_name,
                ),
                (
                    &cache.cached_values_input_name,
                    &cache.cached_values_output_name,
                ),
                (
                    &cache.cached_positions_input_name,
                    &cache.cached_positions_output_name,
                ),
            ] {
                self.state
                    .insert(input.clone(), onnx::take(&mut outputs, output)?);
            }
        }
        let audio = onnx::take(&mut outputs, "audio")?;
        let length = onnx::take(&mut outputs, "audio_lengths")?;
        let (shape, audio) = audio
            .try_extract_tensor::<f32>()
            .map_err(|error| format!("invalid_output: {error}"))?;
        let (_, length) = length
            .try_extract_tensor::<i32>()
            .map_err(|error| format!("invalid_output: {error}"))?;
        if shape.len() != 3
            || shape[0] != 1
            || shape[1] != 2
            || length.len() != 1
            || length[0] < 0
            || i64::from(length[0]) > shape[2]
        {
            return Err("invalid_output: codec returned an invalid stereo chunk".into());
        }
        if length[0] == 0 {
            return Ok(());
        }
        let width = shape[2] as usize;
        let count = length[0] as usize;
        let mono = (0..count)
            .map(|index| (audio[index] + audio[width + index]) * 0.5)
            .collect::<Vec<_>>();
        publisher.publish(&mono, 48_000)
    }
}
