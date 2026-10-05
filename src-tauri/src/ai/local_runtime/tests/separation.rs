use std::io::Write;

use super::*;

#[test]
fn source_separation_artifact_copy_validates_timeline_and_cleans_worker_file() {
    let request_id = uuid::Uuid::new_v4().to_string();
    let root = std::env::temp_dir().join(format!("knovum-runtime-separation-{request_id}"));
    let attempt = root.join(format!("attempt-{request_id}"));
    fs::create_dir_all(&attempt).unwrap();
    let source = attempt.join("background.wav");
    write_wav(&source, 4);
    let payload = fs::read(&source).unwrap();
    let reference = SynthesizedAudioFileReference {
        file_name: format!("attempt-{request_id}/background.wav"),
        media_type: "audio/wav".into(),
        byte_length: payload.len() as u64,
        sha256: format!("{:x}", Sha256::digest(&payload)),
        sample_rate: SAMPLE_RATE,
        channels: CHANNELS,
        sample_format: "pcm_s16le".into(),
    };
    let target = root.join("published.wav");
    let metadata = copy_artifact(
        &root,
        &request_id,
        &reference,
        4,
        u64::MAX,
        &CancellationToken::new(),
        &target,
    )
    .unwrap();
    assert_eq!(metadata.sample_count, 4);
    assert!(target.is_file());
    assert!(!source.exists());

    write_wav(&source, 4);
    let invalid_target = root.join("invalid.wav");
    let error = copy_artifact(
        &root,
        &request_id,
        &reference,
        5,
        u64::MAX,
        &CancellationToken::new(),
        &invalid_target,
    )
    .err()
    .unwrap();
    assert!(error.contains("WAV metadata differs"));
    assert!(!invalid_target.exists());
    fs::remove_dir_all(root).unwrap();
}

fn write_wav(path: &Path, sample_count: u64) {
    let data_bytes = sample_count * u64::from(CHANNELS) * 2;
    let mut header = [0_u8; 44];
    header[0..4].copy_from_slice(b"RIFF");
    header[4..8].copy_from_slice(&u32::try_from(data_bytes + 36).unwrap().to_le_bytes());
    header[8..12].copy_from_slice(b"WAVE");
    header[12..16].copy_from_slice(b"fmt ");
    header[16..20].copy_from_slice(&16_u32.to_le_bytes());
    header[20..22].copy_from_slice(&1_u16.to_le_bytes());
    header[22..24].copy_from_slice(&CHANNELS.to_le_bytes());
    header[24..28].copy_from_slice(&SAMPLE_RATE.to_le_bytes());
    header[28..32].copy_from_slice(&(SAMPLE_RATE * u32::from(CHANNELS) * 2).to_le_bytes());
    header[32..34].copy_from_slice(&(CHANNELS * 2).to_le_bytes());
    header[34..36].copy_from_slice(&16_u16.to_le_bytes());
    header[36..40].copy_from_slice(b"data");
    header[40..44].copy_from_slice(&u32::try_from(data_bytes).unwrap().to_le_bytes());
    let mut file = fs::File::create(path).unwrap();
    file.write_all(&header).unwrap();
    file.write_all(&vec![0; data_bytes as usize]).unwrap();
}
