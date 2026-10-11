use super::{artifacts, credits::Credits};
use crate::{artifact, cancellation::Cancellation, protocol::Output};
use knovum_inference_contracts::speech_wire::*;
use sha2::{Digest, Sha256};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

pub(crate) struct Publisher {
    directory: PathBuf,
    identity: InferenceRequestIdentity,
    credits: Arc<Credits>,
    output: Output,
    cancel: Cancellation,
    seq: u64,
    bytes: u64,
    samples: u64,
    digest: Sha256,
}
impl Publisher {
    pub(crate) fn new(
        work: &Path,
        identity: &InferenceRequestIdentity,
        credits: Arc<Credits>,
        output: &Output,
        cancel: &Cancellation,
    ) -> Result<Self, String> {
        let directory = artifacts::attempt(work, &identity.request_id)?;
        Ok(Self {
            directory,
            identity: identity.clone(),
            credits,
            output: output.clone(),
            cancel: cancel.clone(),
            seq: 0,
            bytes: 0,
            samples: 0,
            digest: Sha256::new(),
        })
    }
    pub(crate) fn publish(&mut self, values: &[f32], rate: u32) -> Result<(), String> {
        self.cancel.check()?;
        if rate != 48_000
            || values.is_empty()
            || values.len() > 4_194_304
            || values.iter().any(|value| !value.is_finite())
        {
            return Err("invalid_output: synthesis returned an invalid mono audio chunk".into());
        }
        let bytes: Vec<u8> = values
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect();
        if self.bytes + bytes.len() as u64 + 44 > 32 * 1024 * 1024 {
            return Err("resource_exhausted: synthesis exceeded its audio budget".into());
        }
        self.seq += 1;
        self.credits
            .reserve(self.seq, bytes.len() as u64, &self.cancel)?;
        let name = format!("chunk-{:08}.pcm", self.seq);
        artifact::write_bytes(&self.directory, &name, &bytes)?;
        let reference = SynthesizedAudioFileReference {
            file_name: format!("attempt-{}/{name}", self.identity.request_id),
            media_type: "application/vnd.knovum.pcm-f32le".into(),
            byte_length: bytes.len() as u64,
            sha256: hex::encode(Sha256::digest(&bytes)),
            sample_rate: rate,
            channels: 1,
            sample_format: "pcm_f32le".into(),
        };
        self.output.notification(
            "speech.chunk",
            &SpeechSynthesisChunkEvent {
                request_id: self.identity.request_id.clone(),
                attempt_epoch: self.identity.attempt_epoch,
                seq: self.seq,
                granularity: SpeechSynthesisChunkGranularity::ModelChunk,
                chunk_ref: reference,
                sample_count: values.len() as u64,
            },
        )?;
        self.digest.update(&bytes);
        self.bytes += bytes.len() as u64;
        self.samples += values.len() as u64;
        Ok(())
    }
    pub(crate) fn finish(self) -> Result<SpeechSynthesis, String> {
        self.credits.wait(&self.cancel)?;
        if self.samples == 0 {
            return Err("invalid_output: synthesis returned no audio".into());
        }
        Ok(SpeechSynthesis {
            request_id: self.identity.request_id,
            attempt_epoch: self.identity.attempt_epoch,
            model_spec_id: self.identity.model_spec_id,
            byte_length: self.bytes,
            sha256: hex::encode(self.digest.finalize()),
            sample_rate: 48_000,
            channels: 1,
            sample_format: "pcm_f32le".into(),
            sample_count: self.samples,
        })
    }
}
