use super::inspect;
use std::io::Cursor;

fn audio() -> Vec<u8> {
    let mut cursor = Cursor::new(Vec::new());
    let mut writer = hound::WavWriter::new(
        &mut cursor,
        hound::WavSpec {
            channels: 2,
            sample_rate: 44_100,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        },
    )
    .unwrap();
    for value in [1_i16, -1, 2, -2] {
        writer.write_sample(value).unwrap();
    }
    writer.finalize().unwrap();
    cursor.into_inner()
}

#[test]
fn metadata_reports_frames_instead_of_interleaved_samples() {
    let bytes = audio();
    let metadata = inspect(Cursor::new(&bytes)).unwrap();
    assert_eq!(metadata.sample_count, 2);
    assert_eq!(metadata.spec.channels, 2);
    assert_eq!(metadata.byte_length, bytes.len() as u64);
}

#[test]
fn truncated_payload_is_rejected_even_when_the_riff_size_is_rewritten() {
    let mut bytes = audio();
    bytes.truncate(bytes.len() - 2);
    let riff_size = (bytes.len() - 8) as u32;
    bytes[4..8].copy_from_slice(&riff_size.to_le_bytes());
    assert!(inspect(Cursor::new(bytes)).is_err());
}

#[test]
fn trailing_bytes_outside_the_declared_container_are_rejected() {
    let mut bytes = audio();
    bytes.push(0);
    assert!(inspect(Cursor::new(bytes)).is_err());
}

#[test]
fn float_samples_preserve_values_and_silence_without_loading_whole_audio() {
    let root = std::env::temp_dir().join(format!("knovum-wav-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&root).unwrap();
    let source = root.join("source.wav");
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: 48_000,
        bits_per_sample: 32,
        sample_format: hound::SampleFormat::Float,
    };
    let mut writer = hound::WavWriter::create(&source, spec).unwrap();
    for sample in [0.25_f32, -0.5] {
        writer.write_sample(sample).unwrap();
    }
    writer.finalize().unwrap();
    let mut cursor = Cursor::new(Vec::new());
    let mut writer = hound::WavWriter::new(&mut cursor, spec).unwrap();
    let token = tokio_util::sync::CancellationToken::new();
    super::write_silence(&mut writer, 2, &token).unwrap();
    super::copy_float_samples(&source, &mut writer, &token).unwrap();
    writer.finalize().unwrap();
    let values = hound::WavReader::new(Cursor::new(cursor.into_inner()))
        .unwrap()
        .into_samples::<f32>()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    assert_eq!(values, vec![0.0, 0.0, 0.25, -0.5]);
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn cancelled_wav_write_stops_before_emitting_samples() {
    let mut cursor = Cursor::new(Vec::new());
    let mut writer = hound::WavWriter::new(
        &mut cursor,
        hound::WavSpec {
            channels: 1,
            sample_rate: 48_000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .unwrap();
    let token = tokio_util::sync::CancellationToken::new();
    token.cancel();
    assert_eq!(
        super::write_silence(&mut writer, 48_000, &token).unwrap_err(),
        "cancelled"
    );
    assert_eq!(writer.duration(), 0);
    writer.finalize().unwrap();
}

#[test]
fn padded_pcm_data_length_is_validated_using_storage_bytes() {
    let mut cursor = Cursor::new(Vec::new());
    let mut writer = hound::WavWriter::new_with_spec_ex(
        &mut cursor,
        hound::WavSpecEx {
            spec: hound::WavSpec {
                channels: 1,
                sample_rate: 48_000,
                bits_per_sample: 24,
                sample_format: hound::SampleFormat::Int,
            },
            bytes_per_sample: 4,
        },
    )
    .unwrap();
    writer.write_sample(100_i32).unwrap();
    writer.finalize().unwrap();
    let mut bytes = cursor.into_inner();
    assert_eq!(inspect(Cursor::new(&bytes)).unwrap().sample_count, 1);
    bytes.pop();
    let riff_size = (bytes.len() - 8) as u32;
    bytes[4..8].copy_from_slice(&riff_size.to_le_bytes());
    assert!(inspect(Cursor::new(bytes)).is_err());
}
