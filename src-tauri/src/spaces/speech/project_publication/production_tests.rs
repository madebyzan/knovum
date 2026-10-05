use super::super::{audio_format::WavPrecision, project_config::parse_audiobook};
use super::{
    encoding::{self, EncodeRequest, NormalizedSource},
    pauses,
};
use std::{fs, path::PathBuf, process::Command};
use tokio_util::sync::CancellationToken;

#[test]
#[ignore = "Requires the explicit FFmpeg test binary"]
fn precision_pause_and_book_metadata_are_in_published_containers() {
    let work = std::env::temp_dir().join(format!("knovum-book-test-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&work).unwrap();
    let tool =
        PathBuf::from(std::env::var_os("KNOVUM_FFMPEG_PATH").expect("Set KNOVUM_FFMPEG_PATH"));
    let master = work.join("master.wav");
    let mut writer = hound::WavWriter::create(
        &master,
        hound::WavSpec {
            channels: 1,
            sample_rate: 24000,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        },
    )
    .unwrap();
    for index in 0..96000 {
        writer
            .write_sample(0.1 * (index as f32 * std::f32::consts::TAU * 440.0 / 24000.0).sin())
            .unwrap();
    }
    writer.finalize().unwrap();
    let sources = [
        NormalizedSource {
            path: master.clone(),
            sample_count: 96000,
        },
        NormalizedSource {
            path: master,
            sample_count: 96000,
        },
    ];
    let cancellation = CancellationToken::new();
    let paused = pauses::insert(&sources, 24000, 250, &work, &cancellation).unwrap();
    assert_eq!(
        paused.iter().map(|source| source.sample_count).sum::<u64>(),
        198000
    );
    let concat = encoding::write_concat_file(&work, &paused).unwrap();
    for (name, precision, bits, format) in [
        ("pcm16", WavPrecision::Pcm16, 16, hound::SampleFormat::Int),
        ("pcm24", WavPrecision::Pcm24, 24, hound::SampleFormat::Int),
        (
            "float32",
            WavPrecision::Float32,
            32,
            hound::SampleFormat::Float,
        ),
    ] {
        let output = work.join(format!("{name}.wav"));
        encoding::encode(
            EncodeRequest {
                ffmpeg: &tool,
                concat_path: &concat,
                output: &output,
                format: "wav",
                sample_rate: 24000,
                wav_precision: precision,
                audiobook: None,
                measurement: None,
                metadata: None,
            },
            &cancellation,
        )
        .unwrap();
        let reader = hound::WavReader::open(output).unwrap();
        assert_eq!(reader.spec().bits_per_sample, bits);
        assert_eq!(reader.spec().sample_format, format);
        assert_eq!(reader.duration(), 198000);
    }
    let cover = work.join("cover.png");
    image::RgbImage::from_pixel(64, 64, image::Rgb([20, 30, 40]))
        .save(&cover)
        .unwrap();
    let cover_revision = crate::file_inspection::content_revision(&cover).unwrap();
    let key = uuid::Uuid::new_v4().to_string();
    let config = parse_audiobook(&serde_json::json!({"format":"m4b","sampleRate":24000,"loudnessLufs":-20,"truePeakDb":-2,"paragraphPauseMs":250,"chapterPauseMs":1000,"wavPrecision":"pcm24","metadata":{"title":"Book = title","author":"Author","narrator":"Narrator","cover":{"assetPath":cover.to_string_lossy(),"sourceRevision":cover_revision}},"chapters":[{"key":key,"title":"Chapter"}],"roles":[]})).unwrap();
    let metadata = encoding::write_chapter_metadata(
        &work,
        &config,
        &[Some(key.clone()), Some(key.clone()), Some(key)],
        &paused,
        24000,
    )
    .unwrap();
    let text = fs::read_to_string(metadata).unwrap();
    assert!(text.contains("title=Book \\= title"));
    assert!(text.contains("album_artist=Author"));
    assert!(text.contains("artist=Narrator"));
    assert!(text.contains("END=198000"));
    let book = work.join("book.m4b");
    encoding::encode(
        EncodeRequest {
            ffmpeg: &tool,
            concat_path: &concat,
            output: &book,
            format: "m4b",
            sample_rate: 24000,
            wav_precision: WavPrecision::Pcm24,
            audiobook: Some(&config),
            measurement: None,
            metadata: Some(&work.join("chapters.ffmetadata.part")),
        },
        &cancellation,
    )
    .unwrap();
    let inspection = Command::new(&tool)
        .args(["-hide_banner", "-nostdin", "-i"])
        .arg(&book)
        .args(["-map", "0:a:0", "-f", "null", "-"])
        .output()
        .unwrap();
    assert!(inspection.status.success());
    let diagnostic = String::from_utf8(inspection.stderr).unwrap();
    assert!(diagnostic.contains("Author"));
    assert!(diagnostic.contains("Narrator"));
    assert!(diagnostic.contains("Book = title"));
    assert!(diagnostic.contains("Chapter"));
    assert!(diagnostic.contains("attached pic"));
    fs::remove_dir_all(work).unwrap();
}
