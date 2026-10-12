use super::super::{
    contracts::{VideoAsset, VideoAssetKind, VideoAudioClip, VideoTransition, VideoTransitionKind},
    filter_graph, render_inputs,
    sources::ResolvedAsset,
    start::VideoEditSource,
    timing, validation,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
use tokio_util::sync::CancellationToken;

struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn run(ffmpeg: &Path, args: &[&str]) -> std::process::Output {
    let output = Command::new(ffmpeg)
        .args(["-nostdin", "-hide_banner", "-loglevel", "error", "-y"])
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

#[test]
#[ignore = "requires KNOVUM_TEST_FFMPEG pointing to the qualified bundled FFmpeg"]
fn renders_real_concat_transitions_images_and_audio_with_the_production_graph() {
    let ffmpeg =
        PathBuf::from(std::env::var("KNOVUM_TEST_FFMPEG").expect("set the qualified FFmpeg path"));
    let directory =
        Fixture(std::env::temp_dir().join(format!("knovum-video-smoke-{}", uuid::Uuid::new_v4())));
    fs::create_dir(&directory.0).unwrap();
    let video = directory
        .0
        .join("source.mp4")
        .to_string_lossy()
        .into_owned();
    let image = directory.0.join("image.png").to_string_lossy().into_owned();
    let music = directory.0.join("music.wav").to_string_lossy().into_owned();
    run(
        &ffmpeg,
        &[
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=160x90:r=30:d=8",
            "-f",
            "lavfi",
            "-i",
            "sine=f=440:r=48000:d=1",
            "-c:v",
            "libx264",
            "-threads",
            "1",
            "-c:a",
            "aac",
            &video,
        ],
    );
    run(
        &ffmpeg,
        &[
            "-f",
            "lavfi",
            "-i",
            "color=c=blue:s=160x90",
            "-frames:v",
            "1",
            "-threads",
            "1",
            &image,
        ],
    );
    run(
        &ffmpeg,
        &["-f", "lavfi", "-i", "sine=f=220:r=48000:d=1", &music],
    );
    let silent = directory
        .0
        .join("silent.mp4")
        .to_string_lossy()
        .into_owned();
    run(
        &ffmpeg,
        &["-i", &video, "-map", "0:v", "-c:v", "copy", "-an", &silent],
    );
    assert!(
        !super::super::source_audio::inspect(&ffmpeg, &silent, &CancellationToken::new()).unwrap()
    );
    let encoder = crate::spaces::derivation::video_output::platform_h264_encoder().unwrap();
    for kind in [
        VideoTransitionKind::Dissolve,
        VideoTransitionKind::WipeLeft,
        VideoTransitionKind::WipeRight,
    ] {
        let mut project = super::project();
        project.output.width = 160;
        project.output.height = 90;
        project.assets[0].width = 160;
        project.assets[0].height = 90;
        project.video_track.clips[1].transition = Some(VideoTransition {
            kind,
            duration_ms: 500,
        });
        let mut still = project.video_track.clips[1].clone();
        still.id = uuid::Uuid::new_v4().to_string();
        still.asset_id = "image".into();
        still.start_ms = 0;
        still.end_ms = 1000;
        still.transition = None;
        project.video_track.clips.push(still);
        project.assets.push(VideoAsset {
            node_id: "image".into(),
            source_revision: "image-revision".into(),
            kind: VideoAssetKind::Image,
            duration_ms: 0,
            width: 160,
            height: 90,
        });
        project.assets.push(VideoAsset {
            node_id: "music".into(),
            source_revision: "music-revision".into(),
            kind: VideoAssetKind::Audio,
            duration_ms: 1000,
            width: 0,
            height: 0,
        });
        project.audio_track.clips.push(VideoAudioClip {
            id: uuid::Uuid::new_v4().to_string(),
            asset_id: "music".into(),
            start_ms: 0,
            end_ms: 1000,
            timeline_start_ms: 500,
            volume: 1.0,
            fade_in_ms: 100,
            fade_out_ms: 100,
        });
        validation::validate_project(&project, "revision").unwrap();
        let source = VideoEditSource {
            project,
            assets: vec![
                ResolvedAsset {
                    node_id: "source".into(),
                    path: video.clone(),
                },
                ResolvedAsset {
                    node_id: "image".into(),
                    path: image.clone(),
                },
                ResolvedAsset {
                    node_id: "music".into(),
                    path: music.clone(),
                },
            ],
        };
        let (inputs, audio) =
            render_inputs::prepare(&ffmpeg, &source, &CancellationToken::new()).unwrap();
        assert_eq!(audio, [true, true, false]);
        let graph = filter_graph::build(&source.project, &audio, None).unwrap();
        let output = directory
            .0
            .join("rendered.mp4")
            .to_string_lossy()
            .into_owned();
        let mut args = inputs.iter().map(String::as_str).collect::<Vec<_>>();
        let duration = timing::seconds(timing::duration_ms(&source.project));
        args.extend([
            "-filter_complex_threads",
            "2",
            "-filter_complex",
            &graph,
            "-map",
            "[outv]",
            "-map",
            "[outa]",
            "-c:v",
            encoder,
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-t",
            &duration,
            &output,
        ]);
        run(&ffmpeg, &args);
        let frames = run(
            &ffmpeg,
            &["-i", &output, "-map", "0:v", "-f", "framemd5", "-"],
        );
        let count = String::from_utf8(frames.stdout)
            .unwrap()
            .lines()
            .filter(|line| line.starts_with("0,"))
            .count();
        assert_eq!(
            count, 165,
            "concat followed by transition must preserve output frame timing"
        );
    }
}
