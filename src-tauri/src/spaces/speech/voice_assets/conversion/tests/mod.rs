use super::*;

#[test]
#[ignore = "Requires the explicit FFmpeg test binary"]
fn converts_only_the_selected_late_clip_to_canonical_pcm() {
    let root = std::env::temp_dir().join(format!("knovum-reference-clip-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&root).unwrap();
    let source = root.join("long-stereo.wav");
    let output = root.join("selected.input");
    let mut writer = hound::WavWriter::create(
        &source,
        hound::WavSpec {
            channels: 2,
            sample_rate: 44_100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .unwrap();
    for frame in 0..38 * 44_100 {
        let sample = if frame < 32 * 44_100 {
            0_i16
        } else {
            4_000_i16
        };
        writer.write_sample(sample).unwrap();
        writer.write_sample(sample).unwrap();
    }
    writer.finalize().unwrap();
    let ffmpeg = std::path::PathBuf::from(
        std::env::var_os("KNOVUM_FFMPEG_PATH").expect("Set KNOVUM_FFMPEG_PATH"),
    );
    convert(
        &ffmpeg,
        &source,
        &output,
        &CancellationToken::new(),
        (32_000, 37_000),
    )
    .unwrap();
    verify_duration(&output, (32_000, 37_000)).unwrap();
    let audio = super::super::reference_metadata(&output).unwrap();
    assert!(audio.byte_length < 161_000);
    assert_eq!(audio.sample_rate, 16_000);
    assert_eq!(audio.channels, 1);
    let mut reader = hound::WavReader::open(&output).unwrap();
    assert_eq!(reader.duration(), 80_000);
    assert!(reader
        .samples::<i16>()
        .skip(1000)
        .take(1000)
        .all(|sample| sample.unwrap() > 3_900));
    std::fs::remove_file(&output).unwrap();
    let cancellation = CancellationToken::new();
    cancellation.cancel();
    assert_eq!(
        convert(&ffmpeg, &source, &output, &cancellation, (32_000, 37_000)).unwrap_err(),
        "cancelled"
    );
    assert!(!output.exists());
    std::fs::remove_dir_all(root).unwrap();
}
