use super::*;
#[test]
#[ignore = "Requires the explicit FFmpeg test binary"]
fn multiple_language_tracks_preserve_duration_and_stream_metadata() {
    let work = std::env::temp_dir().join(format!("knovum-tracks-test-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&work).unwrap();
    let tool =
        PathBuf::from(std::env::var_os("KNOVUM_FFMPEG_PATH").expect("Set KNOVUM_FFMPEG_PATH"));
    let source = work.join("source.mp4");
    assert!(Command::new(&tool)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "color=size=32x32:rate=10:duration=2",
            "-f",
            "lavfi",
            "-i",
            "sine=duration=1",
            "-c:v",
            "mpeg4",
            "-c:a",
            "aac"
        ])
        .arg(&source)
        .status()
        .unwrap()
        .success());
    let audio = work.join("track.wav");
    assert!(Command::new(&tool)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-f",
            "lavfi",
            "-i",
            "sine=duration=2",
            "-c:a",
            "pcm_f32le"
        ])
        .arg(&audio)
        .status()
        .unwrap()
        .success());
    let tracks = [
        Track {
            path: audio.clone(),
            sha256: hash_file(&audio).unwrap(),
            label: "Chinese".into(),
            language: "zh-CN".into(),
        },
        Track {
            path: audio,
            sha256: String::new(),
            label: "English".into(),
            language: "en".into(),
        },
    ];
    let output = work.join("delivery.mp4");
    let cancel = tokio_util::sync::CancellationToken::new();
    mux_tracks(
        TrackMuxRequest {
            ffmpeg: &tool,
            source: &source,
            tracks: &tracks,
            output: &output,
            duration_ms: 2000,
            retain_original_track: true,
        },
        &cancel,
    )
    .unwrap();
    for stream in ["0:v:0", "0:a:0", "0:a:1", "0:a:2"] {
        let duration = policy::verified_duration_us(&tool, &output, stream, 2000, &cancel).unwrap();
        assert!(
            (1_900_000..=2_100_000).contains(&duration),
            "Wrong {stream} duration: {duration}"
        );
    }
    let inspection = Command::new(&tool)
        .args(["-hide_banner", "-nostdin", "-i"])
        .arg(&output)
        .args(["-map", "0:a", "-f", "null", "-"])
        .output()
        .unwrap();
    assert!(inspection.status.success());
    let diagnostic = String::from_utf8(inspection.stderr).unwrap();
    assert!(diagnostic.contains("(zho)"));
    assert!(diagnostic.contains("(eng)"));
    assert!(diagnostic.contains("Original"));
    std::fs::remove_dir_all(work).unwrap();
}
