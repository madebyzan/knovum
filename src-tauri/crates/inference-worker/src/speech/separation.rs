//! HTDemucs source separation.
//!
//! A source of any admitted length is streamed through the model in blocks that overlap their
//! neighbours, and every boundary is cross-faded linearly over the overlap so no block edge is
//! audible. HTDemucs estimates drums, bass, other and vocals; the background is the sum of every
//! stem but vocals, which is how Demucs itself defines its two-stem output.

use super::{
    artifacts,
    audio::{WaveStream, WaveWriter},
    audiocpp::{AudioView, Session, SessionSpec},
};
use crate::{cancellation::Cancellation, initialization::VerifiedRuntime};
use knovum_inference_contracts::{
    execution_profile::separation,
    runtime::RuntimeInitializeParams,
    speech_wire::{
        SpeechSeparateParams, SpeechSeparation, SEPARATION_MAX_DURATION_MS, SEPARATION_SAMPLE_RATE,
    },
};
use std::path::{Path, PathBuf};

const RATE: usize = SEPARATION_SAMPLE_RATE as usize;
const CHANNELS: u16 = 2;
/// The span one model call covers, beside its overlap.
const BLOCK_FRAMES: usize = 60 * RATE;
/// How far a block reaches past each of its boundaries; the cross-fade spans twice this.
const OVERLAP_FRAMES: usize = 4 * RATE;
const STEMS: [&str; 4] = ["drums", "bass", "other", "vocals"];

pub(crate) struct Separator {
    session: Session,
}

/// Admits a separation profile and returns the model file it names. Pure in its initialization
/// message, so an obsolete profile is rejected before the audio.cpp library is loaded.
pub(super) fn admit(params: &RuntimeInitializeParams) -> Result<PathBuf, String> {
    let file = match params.capability_spec_id.as_str() {
        "htdemucs-q8_0-v1" => "htdemucs-q8_0.gguf",
        "htdemucs-f16-v1" => "htdemucs-f16.gguf",
        _ => return Err("unsupported_device: source separation profile is unavailable".into()),
    };
    if separation(&params.capability_spec_id, &params.device_profile.device).as_deref()
        != Some(params.device_profile.execution_profile_id.as_str())
        || !(1..=8).contains(&params.device_profile.cpu_threads)
    {
        return Err("unsupported_device: source separation profile is unavailable".into());
    }
    Ok(PathBuf::from(params.model.snapshot()?).join(file))
}

impl Separator {
    pub(crate) fn new(
        params: &RuntimeInitializeParams,
        runtime: &VerifiedRuntime,
    ) -> Result<Self, String> {
        let model = admit(params)?;
        let session = Session::open(
            runtime,
            SessionSpec {
                model: &model,
                family: c"htdemucs",
                task: c"sep",
                device: &params.device_profile.device,
                threads: params.device_profile.cpu_threads,
                failure: "separation_failed",
            },
        )?;
        Ok(Self { session })
    }

    pub(crate) fn separate(
        &mut self,
        work: &Path,
        params: SpeechSeparateParams,
        cancel: &Cancellation,
    ) -> Result<SpeechSeparation, String> {
        let maximum_frames = (SEPARATION_MAX_DURATION_MS as usize / 1_000) * RATE;
        let mut source = WaveStream::open(
            &params.audio_ref,
            SEPARATION_SAMPLE_RATE,
            CHANNELS,
            maximum_frames,
        )?;
        let total = source.frames();
        let request = &params.identity.request_id;
        let directory = artifacts::attempt(work, request)?;
        let mut vocals = WaveWriter::create(&directory, "vocals.wav", RATE as u32, CHANNELS)?;
        let mut background =
            WaveWriter::create(&directory, "background.wav", RATE as u32, CHANNELS)?;
        let mut mix = Mix::default();
        let mut input = Block::default();
        let blocks = total.div_ceil(BLOCK_FRAMES).max(1);
        for index in 0..blocks {
            cancel.check()?;
            let span = Span::of(index, blocks, total);
            input.read_to(&mut source, span.start, span.end)?;
            let (voice, rest) = self.stems(&input.samples)?;
            mix.add(&span, &voice, &rest);
            let (voice, rest) = mix.settle(span.settled);
            vocals.write(&voice)?;
            background.write(&rest)?;
        }
        source.finish()?;
        cancel.check()?;
        let prefix = format!("attempt-{request}");
        Ok(SpeechSeparation {
            request_id: params.identity.request_id.clone(),
            attempt_epoch: params.identity.attempt_epoch,
            model_spec_id: params.identity.model_spec_id,
            vocals_ref: vocals.finish(&prefix)?,
            background_ref: background.finish(&prefix)?,
            sample_count: total as u64,
        })
    }

    /// Vocals and their background for one block of interleaved stereo.
    fn stems(&self, samples: &[f32]) -> Result<(Vec<f32>, Vec<f32>), String> {
        let mut request = self.session.request()?;
        request.audio(AudioView {
            samples,
            sample_rate: SEPARATION_SAMPLE_RATE,
            channels: CHANNELS,
        })?;
        let output = self.session.run(&request)?;
        let streams = output.named_audio()?;
        if streams.len() != STEMS.len()
            || STEMS
                .iter()
                .any(|stem| streams.iter().filter(|(id, _)| id == stem).count() != 1)
            || streams.iter().any(|(_, audio)| {
                audio.sample_rate != SEPARATION_SAMPLE_RATE
                    || audio.channels != CHANNELS
                    || audio.samples.len() != samples.len()
            })
        {
            return Err("invalid_output: separation stems differ from their source".into());
        }
        let mut voice = Vec::new();
        let mut rest = vec![0.0_f32; samples.len()];
        for (id, audio) in &streams {
            if id == "vocals" {
                voice = audio.samples.to_vec();
            } else {
                for (sum, sample) in rest.iter_mut().zip(audio.samples) {
                    *sum += sample;
                }
            }
        }
        Ok((voice, rest))
    }
}

/// The frames one model call reads, and how its estimate is weighed against its neighbours'.
struct Span {
    start: usize,
    end: usize,
    core_start: usize,
    core_end: usize,
    first: bool,
    last: bool,
    /// Every frame before this is complete once this block has been added.
    settled: usize,
}

impl Span {
    fn of(index: usize, blocks: usize, total: usize) -> Self {
        let core_start = index * BLOCK_FRAMES;
        let core_end = ((index + 1) * BLOCK_FRAMES).min(total);
        let last = index + 1 == blocks;
        Self {
            start: core_start.saturating_sub(OVERLAP_FRAMES),
            end: (core_end + OVERLAP_FRAMES).min(total),
            core_start,
            core_end,
            first: index == 0,
            last,
            settled: if last {
                total
            } else {
                core_end - OVERLAP_FRAMES
            },
        }
    }

    /// The share of `frame` this block's estimate contributes: linear ramps across each overlap,
    /// complementary to the neighbour's, so every frame's shares sum to one.
    fn weight(&self, frame: usize) -> f32 {
        let span = (2 * OVERLAP_FRAMES) as f32;
        let rise = if self.first {
            1.0
        } else {
            ((frame + OVERLAP_FRAMES - self.core_start) as f32 + 0.5) / span
        };
        let fall = if self.last {
            1.0
        } else {
            ((self.core_end + OVERLAP_FRAMES - frame) as f32 - 0.5) / span
        };
        rise.min(fall).clamp(0.0, 1.0)
    }
}

/// The source frames the current block reads, carried over from the previous block where they
/// overlap.
#[derive(Default)]
struct Block {
    start: usize,
    samples: Vec<f32>,
}

impl Block {
    fn read_to(&mut self, source: &mut WaveStream, start: usize, end: usize) -> Result<(), String> {
        let channels = usize::from(CHANNELS);
        self.samples.drain(..(start - self.start) * channels);
        self.start = start;
        let missing = end - (start + self.samples.len() / channels);
        if source.read(missing, &mut self.samples)? != missing {
            return Err("invalid_input: separation source ended early".into());
        }
        Ok(())
    }
}

/// Weighted stem estimates for the frames not yet written, from `start` on.
#[derive(Default)]
struct Mix {
    start: usize,
    voice: Vec<f32>,
    rest: Vec<f32>,
}

impl Mix {
    fn add(&mut self, span: &Span, voice: &[f32], rest: &[f32]) {
        let channels = usize::from(CHANNELS);
        let length = (span.end - self.start) * channels;
        self.voice.resize(length, 0.0);
        self.rest.resize(length, 0.0);
        for frame in span.start..span.end {
            let weight = span.weight(frame);
            let source = (frame - span.start) * channels;
            let target = (frame - self.start) * channels;
            for channel in 0..channels {
                self.voice[target + channel] += weight * voice[source + channel];
                self.rest[target + channel] += weight * rest[source + channel];
            }
        }
    }

    /// Hands out every frame before `end`, which no later block contributes to.
    fn settle(&mut self, end: usize) -> (Vec<f32>, Vec<f32>) {
        let count = (end - self.start) * usize::from(CHANNELS);
        self.start = end;
        (
            self.voice.drain(..count).collect(),
            self.rest.drain(..count).collect(),
        )
    }
}

#[cfg(test)]
#[path = "tests/separation.rs"]
mod tests;
