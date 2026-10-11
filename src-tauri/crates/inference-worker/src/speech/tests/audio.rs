use super::super::audio::*;

#[test]
fn wav_decode_binds_header_hash_and_declared_timeline() {
    let work = std::env::temp_dir().join(format!("knovum-audio-{}", std::process::id()));
    std::fs::create_dir(&work).unwrap();
    let samples = vec![0.0, 0.5, -1.0, 0.25];
    let reference = write_wave(&work, "sample.wav", 16_000, 1, &samples).unwrap();
    let input = knovum_inference_contracts::speech_wire::AudioFileReference {
        path: work.join("sample.wav").to_string_lossy().into(),
        media_type: reference.media_type,
        byte_length: reference.byte_length,
        sha256: reference.sha256,
        sample_rate: 16_000,
        channels: 1,
        sample_format: "pcm_s16le".into(),
    };
    let cancel = crate::cancellation::Cancellation::new(u64::MAX);
    let decoded = read(&input, 16_000, 1, 100, &cancel).unwrap();
    assert_eq!(decoded.samples, samples);
    assert_eq!(decoded.frames(), 4);
    assert!(read(&input, 44_100, 1, 100, &cancel).is_err());
    assert!(read(&input, 16_000, 1, 3, &cancel).is_err());
    let mut stale = input;
    stale.sha256 = "0".repeat(64);
    assert!(read(&stale, 16_000, 1, 100, &cancel).is_err());
    std::fs::remove_dir_all(work).unwrap();
}

#[test]
fn pcm_conversion_preserves_half_even_rounding_and_saturates() {
    assert_eq!(pcm16(0.5 / 32768.0), 0);
    assert_eq!(pcm16(1.5 / 32768.0), 2);
    assert_eq!(pcm16(-1.5 / 32768.0), -2);
    assert_eq!(pcm16(2.0), 32767);
    assert_eq!(pcm16(-2.0), -32768);
}
