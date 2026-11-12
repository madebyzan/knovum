//! The transcription capability: one worker freezes one engine at initialization. Whisper decodes
//! with word timestamps; Qwen3-ASR transcribes speech regions and leaves word timing to the
//! separate forced-alignment stage.

use super::{qwen3_asr::Qwen3Asr, whisper::Whisper};
use crate::{cancellation::Cancellation, initialization::VerifiedRuntime, protocol::Output};
use knovum_inference_contracts::{
    runtime::RuntimeInitializeParams,
    speech_wire::{SpeechTranscribeParams, SpeechTranscription},
};
use std::path::Path;

pub(crate) enum Transcriber {
    Whisper(Box<Whisper>),
    Qwen3Asr(Box<Qwen3Asr>),
}

impl Transcriber {
    pub(crate) fn new(
        params: &RuntimeInitializeParams,
        runtime: &VerifiedRuntime,
    ) -> Result<Self, String> {
        if params.capability_spec_id.starts_with("qwen3-asr-1.7b-") {
            Ok(Self::Qwen3Asr(Box::new(Qwen3Asr::new(params, runtime)?)))
        } else {
            Ok(Self::Whisper(Box::new(Whisper::new(params, runtime)?)))
        }
    }

    pub(crate) fn transcribe(
        &mut self,
        work: &Path,
        params: SpeechTranscribeParams,
        cancel: &Cancellation,
        output: &Output,
    ) -> Result<SpeechTranscription, String> {
        match self {
            Self::Whisper(engine) => engine.transcribe(work, params, cancel, output),
            Self::Qwen3Asr(engine) => engine.transcribe(work, params, cancel, output),
        }
    }
}
