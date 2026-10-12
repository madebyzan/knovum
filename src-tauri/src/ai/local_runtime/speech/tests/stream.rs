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
    let pcm: Vec<u8> = [0.0_f32, 1.25]
        .into_iter()
        .flat_map(f32::to_le_bytes)
        .collect();
    let file_name = "chunk-00000001.pcm";
    fs::write(attempt.join(file_name), &pcm).expect("chunk should be written");
    let mut event = SpeechSynthesisChunkEvent {
        request_id: request_id.clone(),
        attempt_epoch: 4,
        seq: 2,
        granularity: SpeechSynthesisChunkGranularity::ModelChunk,
        chunk_ref: SynthesizedAudioFileReference {
            file_name: format!("attempt-{request_id}/{file_name}"),
            media_type: "application/vnd.knovum.pcm-f32le".into(),
            byte_length: pcm.len() as u64,
            sha256: hex::encode(Sha256::digest(&pcm)),
            sample_rate: 48_000,
            channels: 1,
            sample_format: "pcm_f32le".into(),
        },
        sample_count: 2,
    };
    let mut stream = SynthesisStream::new();
    assert!(consume_synthesis_chunk(&mut stream, &event, &root.0, &request_id, 4, None,).is_err());
    assert!(attempt.join(file_name).is_file());

    event.seq = 1;
    let mut callback = |seq, sample_rate, sample_count, bytes: &[u8]| {
        assert_eq!((seq, sample_rate, sample_count), (1, 48_000, 2));
        assert_eq!(bytes, &[0, 0, 255, 127]);
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
        8
    );
    assert!(!attempt.join(file_name).exists());

    let identity = InferenceRequestIdentity {
        request_id: request_id.clone(),
        owner_kind: "speech_preview".into(),
        owner_id: "voice-1".into(),
        attempt_id: request_id.clone(),
        attempt_epoch: 4,
        input_revision: "input-revision".into(),
        model_spec_id: "voxcpm2-2b-gguf-orig-v1".into(),
        budget_ms: u64::MAX,
    };
    let synthesis = SpeechSynthesis {
        request_id,
        attempt_epoch: 4,
        model_spec_id: identity.model_spec_id.clone(),
        byte_length: 8,
        sha256: hex::encode(Sha256::digest(&pcm)),
        sample_rate: 48_000,
        channels: 1,
        sample_format: "pcm_f32le".into(),
        sample_count: 2,
    };
    let audio = finish_synthesis(synthesis, &root.0, &identity, Some(stream))
        .expect("matching summary should finish synthesis");
    assert!(!attempt.exists());
    assert_eq!(audio.sample_count, 2);
    assert_eq!(audio.sample_rate, 48_000);
    assert_eq!(&audio.bytes[..4], b"RIFF");
    assert_eq!(&audio.bytes[44..], pcm);
}
