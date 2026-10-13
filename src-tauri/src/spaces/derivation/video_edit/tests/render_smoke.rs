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

fn fixtures(ffmpeg: &Path, directory: &Path) -> (String, String, String) {
    let video = directory.join("source.mp4").to_string_lossy().into_owned();
    let image = directory.join("image.png").to_string_lossy().into_owned();
    let music = directory.join("music.wav").to_string_lossy().into_owned();
    run(
        ffmpeg,
        &[
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=160x90:r=30:d=8",
            "-f",
            "lavfi",
            "-i",
            "sine=f=440:r=48000:d=8",
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
        ffmpeg,
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
        ffmpeg,
        &["-f", "lavfi", "-i", "sine=f=220:r=48000:d=1", &music],
    );
    (video, image, music)
}

#[test]
#[ignore = "requires KNOVUM_TEST_FFMPEG pointing to the qualified bundled FFmpeg"]
fn renders_transitions_holes_images_and_audio_with_the_production_graph() {
    let ffmpeg =
        PathBuf::from(std::env::var("KNOVUM_TEST_FFMPEG").expect("set the qualified FFmpeg path"));
    let directory =
        Fixture(std::env::temp_dir().join(format!("knovum-video-smoke-{}", uuid::Uuid::new_v4())));
    fs::create_dir(&directory.0).unwrap();
    let (video, image, music) = fixtures(&ffmpeg, &directory.0);
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
        VideoTransitionKind::SlideUp,
    ] {
        let mut project = super::project();
        project.output.width = 160;
        project.output.height = 90;
        project.assets[0].width = 160;
        project.assets[0].height = 90;
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
        // Transition over the first cut, a one-second hole, then a still image.
        project.video_track.clips[1].timeline_start_ms = 1500;
        project.transitions.push(VideoTransition {
            id: uuid::Uuid::new_v4().to_string(),
            from_clip_id: project.video_track.clips[0].id.clone(),
            kind,
            duration_ms: 500,
        });
        let mut still = project.video_track.clips[1].clone();
        still.id = uuid::Uuid::new_v4().to_string();
        still.asset_id = "image".into();
        still.source_in_ms = 0;
        still.source_out_ms = 1000;
        still.timeline_start_ms = 5500;
        still.transform.scale = 1.5;
        still.transform.rotation = 12.0;
        still.transform.x = 0.1;
        project.video_track.clips.push(still);
        project.audio_track.clips.push(VideoAudioClip {
            id: uuid::Uuid::new_v4().to_string(),
            asset_id: "music".into(),
            source_in_ms: 0,
            source_out_ms: 1000,
            timeline_start_ms: 500,
            lane: 0,
            volume: 1.0,
            fade_in_ms: 100,
            fade_out_ms: 100,
        });
        validation::validate_project(&project, "revision").unwrap();
        assert_eq!(timing::duration_ms(&project), 6500);
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
            count, 195,
            "transitions, holes and stills must preserve output frame timing"
        );
    }
}
