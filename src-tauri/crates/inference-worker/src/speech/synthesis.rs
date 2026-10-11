use super::{credits::Credits, moss::Moss, stream::Publisher, voxcpm::VoxCpm};
use crate::{cancellation::Cancellation, protocol::Output};
use knovum_inference_contracts::{
    runtime::{RuntimeAckChunkParams, RuntimeInitializeParams},
    speech_wire::{SpeechSynthesis, SpeechSynthesisEngine, SpeechSynthesizeParams},
};
use std::{
    path::Path,
    sync::{Arc, Mutex},
};

#[derive(Default)]
pub(crate) struct Acknowledgements(Mutex<Option<Arc<Credits>>>);
impl Acknowledgements {
    pub(crate) fn acknowledge(&self, params: &RuntimeAckChunkParams) -> Result<(), String> {
        self.0
            .lock()
            .map_err(|_| "worker_crashed: synthesis acknowledgement lock poisoned")?
            .as_ref()
            .ok_or("runtime_mismatch: no synthesis request owns chunk credits")?
            .acknowledge(params)
    }
}
pub(crate) enum Synthesizer {
    Moss(Box<Moss>),
    Vox(Box<VoxCpm>),
}
impl Synthesizer {
    pub(crate) fn new(params: &RuntimeInitializeParams) -> Result<Self, String> {
        match params.capability_spec_id.as_str() {
            "moss-tts-nano-100m-onnx-native-v2" => Ok(Self::Moss(Box::new(Moss::new(params)?))),
            "voxcpm2-2b-gguf-orig-v1" | "voxcpm2-2b-gguf-q8_0-v1" => {
                Ok(Self::Vox(Box::new(VoxCpm::new(params)?)))
            }
            _ => Err("runtime_mismatch: synthesis model is unsupported".into()),
        }
    }
    pub(crate) fn synthesize(
        &mut self,
        work: &Path,
        params: SpeechSynthesizeParams,
        cancel: &Cancellation,
        output: &Output,
        acks: &Acknowledgements,
    ) -> Result<SpeechSynthesis, String> {
        if params.text.trim().is_empty()
            || params.text.len() > 16_384
            || params.seed > u64::from(u32::MAX)
            || params.profile.sample_rate != 48_000
            || params.profile.speed != 1.0
        {
            return Err("invalid_input: synthesis requires bounded text, a 32-bit seed and the 48 kHz normal-speed profile".into());
        }
        if !matches!(
            (&*self, &params.profile.engine),
            (Self::Moss(_), SpeechSynthesisEngine::MossNano)
                | (Self::Vox(_), SpeechSynthesisEngine::VoxCpm2)
        ) {
            return Err("runtime_mismatch: synthesis engine differs from its model".into());
        }
        let credits = Arc::new(Credits::new(
            &params.identity.request_id,
            params.identity.attempt_epoch,
        ));
        *acks
            .0
            .lock()
            .map_err(|_| "worker_crashed: synthesis acknowledgement lock poisoned")? =
            Some(credits.clone());
        let synthesis = (|| {
            let mut publisher = Publisher::new(work, &params.identity, credits, output, cancel)?;
            match self {
                Self::Moss(engine) => engine.generate(&params, &mut publisher, cancel)?,
                Self::Vox(engine) => engine.generate(&params, &mut publisher, cancel)?,
            }
            cancel.check()?;
            publisher.finish()
        })();
        acks.0
            .lock()
            .map_err(|_| "worker_crashed: synthesis acknowledgement lock poisoned")?
            .take();
        synthesis
    }
}
