mod codec;
mod generation;
mod metadata;
use super::stream::Publisher;
use crate::{artifact, cancellation::Cancellation, onnx};
use codec::Codec;
use knovum_inference_contracts::{
    execution_profile,
    runtime::RuntimeInitializeParams,
    speech_wire::{SpeechSynthesizeParams, VoiceSourceKind},
};
use ort::session::Session;
use sentencepiece::SentencePieceProcessor;
use std::{io::Read, path::PathBuf};

pub(crate) struct Moss {
    params: RuntimeInitializeParams,
    loaded: Option<Loaded>,
}
struct Loaded {
    manifest: metadata::Manifest,
    tokenizer: SentencePieceProcessor,
    prefill: Session,
    decode: Session,
    frame: Session,
    codec: Codec,
}
const VOICES: [&str; 10] = [
    "Junhao", "Zhiming", "Weiguo", "Xiaoyu", "Yuewen", "Lingyu", "Ava", "Bella", "Adam", "Nathan",
];

impl Moss {
    pub(super) fn new(params: &RuntimeInitializeParams) -> Result<Self, String> {
        if params.device_profile.device != "cpu"
            || params.capability_spec_id != "moss-tts-nano-100m-onnx-bundled-v2"
            || !(1..=8).contains(&params.device_profile.cpu_threads)
            || params.model_paths.speech_synthesis.is_none()
            || params.device_profile.execution_profile_id
                != execution_profile::moss_nano(params.device_profile.cpu_threads)
        {
            return Err("unsupported_device: MOSS execution profile is unavailable".into());
        }
        Ok(Self {
            params: params.clone(),
            loaded: None,
        })
    }
    fn load(&mut self) -> Result<&mut Loaded, String> {
        if self.loaded.is_none() {
            let path = PathBuf::from(
                self.params
                    .model_paths
                    .speech_synthesis
                    .as_ref()
                    .ok_or("model_missing: synthesis model is missing")?,
            );
            let manifest: metadata::Manifest = metadata::read(&path, "browser_poc_manifest.json")?;
            let codec: metadata::CodecMeta = metadata::read(&path, "codec_browser_onnx_meta.json")?;
            if !VOICES.iter().all(|name| {
                manifest
                    .builtin_voices
                    .iter()
                    .any(|voice| voice.voice == *name)
            }) {
                return Err("model_corrupt: MOSS preset inventory is incomplete".into());
            }
            let mut bytes = Vec::new();
            artifact::open_regular(&path.join("tokenizer.model"))?
                .take(1_048_577)
                .read_to_end(&mut bytes)
                .map_err(|error| error.to_string())?;
            if bytes.len() > 1_048_576 {
                return Err("model_corrupt: MOSS tokenizer exceeds its byte budget".into());
            }
            let tokenizer = SentencePieceProcessor::from_serialized_proto(&bytes)
                .map_err(|error| format!("model_corrupt: {error}"))?;
            let load =
                |name| onnx::session(&path.join(name), self.params.device_profile.cpu_threads);
            self.loaded = Some(Loaded {
                manifest,
                tokenizer,
                prefill: load("moss_tts_prefill.onnx")?,
                decode: load("moss_tts_decode_step.onnx")?,
                frame: load("moss_tts_local_fixed_sampled_frame.onnx")?,
                codec: Codec::new(
                    load("moss_audio_tokenizer_encode.onnx")?,
                    load("moss_audio_tokenizer_decode_step.onnx")?,
                    codec.streaming_decode,
                )?,
            });
        }
        self.loaded
            .as_mut()
            .ok_or("worker_crashed: MOSS engine is unavailable".into())
    }
    pub(super) fn generate(
        &mut self,
        params: &SpeechSynthesizeParams,
        publisher: &mut Publisher,
        cancel: &Cancellation,
    ) -> Result<(), String> {
        let voice = &params.voice_revision;
        if params.style.is_some() || voice.design_prompt.is_some() || voice.reference_text.is_some()
        {
            return Err(
                "invalid_input: MOSS does not accept design, style or transcript conditioning"
                    .into(),
            );
        }
        let loaded = self.load()?;
        cancel.check()?;
        let codes = match voice.source_kind {
            VoiceSourceKind::Preset => {
                let id = voice
                    .speaker_id
                    .filter(|&id| id < 10)
                    .ok_or("invalid_input: MOSS preset speaker is invalid")?;
                if voice.reference_audio.is_some() {
                    return Err("invalid_input: MOSS preset cannot include reference audio".into());
                }
                loaded
                    .manifest
                    .builtin_voices
                    .iter()
                    .find(|voice| voice.voice == VOICES[id as usize])
                    .ok_or("model_corrupt: MOSS preset is missing")?
                    .prompt_audio_codes
                    .clone()
            }
            VoiceSourceKind::Reference => {
                if voice.speaker_id.is_some() {
                    return Err(
                        "invalid_input: reference voice cannot include a preset speaker".into(),
                    );
                }
                loaded.codec.reference(
                    voice
                        .reference_audio
                        .as_ref()
                        .ok_or("invalid_input: immutable reference audio is missing")?,
                    cancel,
                )?
            }
            VoiceSourceKind::Designed => {
                return Err("invalid_input: MOSS does not support voice design".into())
            }
        };
        if codes.is_empty()
            || codes.len() > 512
            || codes
                .iter()
                .any(|row| row.len() != 16 || row.iter().any(|token| !(0..1024).contains(token)))
        {
            return Err(
                "model_corrupt: MOSS reference codes differ from the frozen profile".into(),
            );
        }
        let text = loaded
            .tokenizer
            .encode(&params.text)
            .map_err(|error| format!("invalid_input: {error}"))?
            .into_iter()
            .map(|piece| piece.id as i32)
            .collect::<Vec<_>>();
        if text.is_empty() || text.len() > 2048 {
            return Err("invalid_input: MOSS text exceeds its token budget".into());
        }
        loaded.codec.reset()?;
        generation::generate(loaded, &codes, &text, params.seed, publisher, cancel)
    }
}
