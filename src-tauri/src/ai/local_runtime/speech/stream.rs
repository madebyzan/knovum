use std::{fs, path::Path};

use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;

use super::super::admission::Priority;
use super::{artifact::read_artifact, validate_identity};
use crate::ai::contracts::speech_wire::{
    InferenceRequestIdentity, SpeechSynthesisChunkEvent, SpeechSynthesisChunkGranularity,
    SpeechSynthesizeResult,
};

const MAX_AUDIO_BYTES: u64 = 16 * 1024 * 1024;

pub(in crate::ai::local_runtime) struct SynthesisStream {
    pcm: Vec<u8>,
    digest: Sha256,
    next_seq: u64,
    sample_rate: Option<u32>,
    granularity: Option<SpeechSynthesisChunkGranularity>,
}

impl SynthesisStream {
    pub(super) fn new() -> Self {
        Self {
            pcm: Vec::new(),
            digest: Sha256::new(),
            next_seq: 1,
            sample_rate: None,
            granularity: None,
        }
    }
}

pub(crate) struct SynthesizedAudio {
    pub bytes: Vec<u8>,
    pub sample_rate: u32,
    pub sample_count: u64,
    pub sha256: String,
}

pub(crate) type SynthesisChunkCallback<'a> =
    dyn FnMut(u64, u32, u64, &[u8]) -> Result<(), String> + Send + 'a;

pub(crate) struct SynthesisExecution<'a> {
    pub cancellation: &'a CancellationToken,
    pub priority: Priority,
    pub on_chunk: Option<&'a mut SynthesisChunkCallback<'a>>,
}

pub(super) fn finish_synthesis(
    result: SpeechSynthesizeResult,
    work_directory: &Path,
    identity: &InferenceRequestIdentity,
    stream: Option<SynthesisStream>,
) -> Result<SynthesizedAudio, String> {
    validate_identity(
        &result.request_id,
        result.attempt_epoch,
        &result.model_spec_id,
        identity,
    )?;
    let stream = stream.ok_or("invalid_output: synthesis returned no chunk stream")?;
    let byte_length = u64::try_from(stream.pcm.len())
        .map_err(|_| "invalid_output: synthesized audio exceeds this platform")?;
    let sample_rate = stream
        .sample_rate
        .ok_or("invalid_output: synthesis returned no audio chunks")?;
    let digest = format!("{:x}", stream.digest.finalize());
    if result.byte_length != byte_length
        || result.sha256 != digest
        || result.sample_rate != sample_rate
        || result.channels != 1
        || result.sample_format != "pcm_s16le"
        || result.sample_count == 0
        || result.sample_count.checked_mul(2) != Some(byte_length)
    {
        return Err("invalid_output: synthesized audio summary differs from its chunks".into());
    }
    let attempt_directory = work_directory.join(format!("attempt-{}", identity.request_id));
    fs::remove_dir(&attempt_directory).map_err(|error| format!("invalid_output: {error}"))?;
    let bytes = wave_from_pcm(&stream.pcm, sample_rate)?;
    Ok(SynthesizedAudio {
        sha256: format!("{:x}", Sha256::digest(&bytes)),
        bytes,
        sample_rate,
        sample_count: result.sample_count,
    })
}

fn wave_from_pcm(pcm: &[u8], sample_rate: u32) -> Result<Vec<u8>, String> {
    let data_size = u32::try_from(pcm.len())
        .map_err(|_| "resource_exhausted: synthesized audio exceeds WAV limits")?;
    let riff_size = data_size
        .checked_add(36)
        .ok_or("resource_exhausted: synthesized audio exceeds WAV limits")?;
    let byte_rate = sample_rate
        .checked_mul(2)
        .ok_or("invalid_output: synthesis sample rate is invalid")?;
    let mut output = Vec::with_capacity(pcm.len() + 44);
    output.extend_from_slice(b"RIFF");
    output.extend_from_slice(&riff_size.to_le_bytes());
    output.extend_from_slice(b"WAVEfmt ");
    output.extend_from_slice(&16_u32.to_le_bytes());
    output.extend_from_slice(&1_u16.to_le_bytes());
    output.extend_from_slice(&1_u16.to_le_bytes());
    output.extend_from_slice(&sample_rate.to_le_bytes());
    output.extend_from_slice(&byte_rate.to_le_bytes());
    output.extend_from_slice(&2_u16.to_le_bytes());
    output.extend_from_slice(&16_u16.to_le_bytes());
    output.extend_from_slice(b"data");
    output.extend_from_slice(&data_size.to_le_bytes());
    output.extend_from_slice(pcm);
    Ok(output)
}

pub(super) fn consume_synthesis_chunk(
    stream: &mut SynthesisStream,
    event: &SpeechSynthesisChunkEvent,
    work_directory: &Path,
    request_id: &str,
    attempt_epoch: u64,
    on_chunk: Option<&mut SynthesisChunkCallback<'_>>,
) -> Result<u64, String> {
    if event.request_id != request_id
        || event.attempt_epoch != attempt_epoch
        || event.seq != stream.next_seq
        || event.chunk_ref.media_type != "application/vnd.knovum.pcm-s16le"
        || event.chunk_ref.channels != 1
        || event.chunk_ref.sample_format != "pcm_s16le"
        || event.chunk_ref.sample_rate == 0
        || event.sample_count == 0
        || event.sample_count.checked_mul(2) != Some(event.chunk_ref.byte_length)
        || stream
            .sample_rate
            .is_some_and(|sample_rate| sample_rate != event.chunk_ref.sample_rate)
        || stream
            .granularity
            .is_some_and(|granularity| granularity != event.granularity)
    {
        return Err("invalid_output: synthesis chunk metadata differs".into());
    }
    let aggregate = u64::try_from(stream.pcm.len())
        .ok()
        .and_then(|size| size.checked_add(event.chunk_ref.byte_length))
        .and_then(|size| size.checked_add(44))
        .ok_or("resource_exhausted: synthesized audio size overflow")?;
    if aggregate > MAX_AUDIO_BYTES {
        return Err("resource_exhausted: synthesized audio exceeds its byte limit".into());
    }
    let bytes = read_artifact(
        work_directory,
        request_id,
        &event.chunk_ref.file_name,
        event.chunk_ref.byte_length,
        &event.chunk_ref.sha256,
        MAX_AUDIO_BYTES,
        false,
    )?;
    if let Some(callback) = on_chunk {
        callback(
            event.seq,
            event.chunk_ref.sample_rate,
            event.sample_count,
            &bytes,
        )?;
    }
    stream.digest.update(&bytes);
    stream.pcm.extend_from_slice(&bytes);
    stream.sample_rate = Some(event.chunk_ref.sample_rate);
    stream.granularity = Some(event.granularity);
    stream.next_seq += 1;
    Ok(event.chunk_ref.byte_length)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::ai::contracts::speech_wire::SynthesizedAudioFileReference;

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new() -> Self {
            let path =
                std::env::temp_dir().join(format!("knovum-speech-stream-{}", uuid::Uuid::new_v4()));
            fs::create_dir(&path).expect("test root should be created");
            Self(path)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn validated_chunks_advance_in_order_and_finish_removes_attempt_directory() {
        let root = TestDirectory::new();
        let request_id = uuid::Uuid::new_v4().to_string();
        let attempt = root.0.join(format!("attempt-{request_id}"));
        fs::create_dir(&attempt).expect("attempt directory should be created");
        let pcm = vec![0_u8, 0, 0xff, 0x7f];
        let file_name = "chunk-00000001.pcm";
        fs::write(attempt.join(file_name), &pcm).expect("chunk should be written");
        let mut event = SpeechSynthesisChunkEvent {
            request_id: request_id.clone(),
            attempt_epoch: 4,
            seq: 2,
            granularity: SpeechSynthesisChunkGranularity::ModelChunk,
            chunk_ref: SynthesizedAudioFileReference {
                file_name: format!("attempt-{request_id}/{file_name}"),
                media_type: "application/vnd.knovum.pcm-s16le".into(),
                byte_length: pcm.len() as u64,
                sha256: format!("{:x}", Sha256::digest(&pcm)),
                sample_rate: 48_000,
                channels: 1,
                sample_format: "pcm_s16le".into(),
            },
            sample_count: 2,
        };
        let mut stream = SynthesisStream::new();
        assert!(
            consume_synthesis_chunk(&mut stream, &event, &root.0, &request_id, 4, None,).is_err()
        );
        assert!(attempt.join(file_name).is_file());

        event.seq = 1;
        let mut callback = |seq, sample_rate, sample_count, bytes: &[u8]| {
            assert_eq!((seq, sample_rate, sample_count), (1, 48_000, 2));
            assert_eq!(bytes, pcm);
            Ok(())
        };
        assert_eq!(
            consume_synthesis_chunk(
                &mut stream,
                &event,
                &root.0,
                &request_id,
                4,
                Some(&mut callback),
            )
            .expect("valid chunk should be consumed"),
            4
        );
        assert!(!attempt.join(file_name).exists());

        let identity = InferenceRequestIdentity {
            request_id: request_id.clone(),
            owner_kind: "speech_preview".into(),
            owner_id: "voice-1".into(),
            attempt_id: request_id.clone(),
            attempt_epoch: 4,
            input_revision: "input-revision".into(),
            model_spec_id: "voxcpm2-2b-v2_0".into(),
            deadline_ms: u64::MAX,
        };
        let result = SpeechSynthesizeResult {
            request_id,
            attempt_epoch: 4,
            model_spec_id: identity.model_spec_id.clone(),
            byte_length: 4,
            sha256: format!("{:x}", Sha256::digest(&pcm)),
            sample_rate: 48_000,
            channels: 1,
            sample_format: "pcm_s16le".into(),
            sample_count: 2,
        };
        let audio = finish_synthesis(result, &root.0, &identity, Some(stream))
            .expect("matching summary should finish synthesis");
        assert!(!attempt.exists());
        assert_eq!(audio.sample_count, 2);
        assert_eq!(audio.sample_rate, 48_000);
        assert_eq!(&audio.bytes[..4], b"RIFF");
        assert_eq!(&audio.bytes[44..], pcm);
    }
}
