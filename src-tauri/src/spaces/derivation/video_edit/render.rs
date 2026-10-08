use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::{ass, filter_graph, report, start::VideoEditSource, VideoEditProgress, CANCELLED};
use crate::{
    local_resources::tools,
    spaces::{
        assets::asset_directory,
        derivation::{
            scratch_file::ScratchFile,
            video_output::{self, RenderedVideo},
        },
    },
    subprocess,
};

pub fn render(
    app: &tauri::AppHandle,
    source: VideoEditSource,
    progress: &Channel<VideoEditProgress>,
    cancellation: &CancellationToken,
) -> Result<RenderedVideo, String> {
    report(progress, "preparingVideoEdit", 8)?;
    let ffmpeg = tools::resolve_ffmpeg(app)?.ok_or("FFmpeg is required to render a video edit")?;
    let directory = asset_directory(app)?;
    let output = ScratchFile::new(directory.join(format!("edited-{}.mp4", Uuid::new_v4())));
    let has_audio = source_has_audio(&ffmpeg, &source.path);
    let mut project = source.project.clone();
    let first_source_ms = project
        .video_track
        .clips
        .iter()
        .map(|clip| clip.start_ms)
        .min()
        .ok_or("Video edit has no clips")?;
    let output_duration_ms: u64 = project
        .video_track
        .clips
        .iter()
        .map(|clip| clip.end_ms - clip.start_ms)
        .sum();
    for clip in &mut project.video_track.clips {
        clip.start_ms -= first_source_ms;
        clip.end_ms -= first_source_ms;
    }
    let subtitle = if project.text_track.items.is_empty() {
        None
    } else {
        let file = ScratchFile::new(directory.join(format!("video-edit-{}.ass", Uuid::new_v4())));
        fs::write(file.path(), ass::render(&project)).map_err(|error| error.to_string())?;
        Some(file)
    };
    let subtitle_name = subtitle
        .as_ref()
        .map(|file| {
            file.path()
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or("Video edit subtitle path is invalid")
        })
        .transpose()?;
    let filter = filter_graph::build(&project, has_audio, subtitle_name)?;
    let source_seek = format!("{:.3}", first_source_ms as f64 / 1000.0);
    report(progress, "renderingVideoEdit", 20)?;
    let mut args = vec![
        "-hide_banner",
        "-loglevel",
        "error",
        "-stats_period",
        "0.5",
        "-progress",
        "pipe:1",
        "-y",
        "-ss",
        &source_seek,
        "-i",
        &source.path,
        "-filter_complex",
        &filter,
        "-map",
        "[outv]",
    ];
    if has_audio {
        args.extend(["-map", "[outa]", "-c:a", "aac", "-b:a", "192k"]);
    }
    let encoder = video_output::platform_h264_encoder()?;
    args.extend([
        "-c:v",
        encoder,
        "-b:v",
        "8M",
        "-maxrate",
        "12M",
        "-bufsize",
        "16M",
        "-pix_fmt",
        "yuv420p",
        "-movflags",
        "+faststart",
    ]);
    let child = Command::new(ffmpeg)
        .args(args)
        .arg(output.path())
        .current_dir(directory)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to launch FFmpeg: {error}"))?;
    let run = subprocess::wait_with_output(
        child,
        cancellation,
        CANCELLED,
        |line| {
            if let Some(processed_us) = line
                .strip_prefix("out_time_ms=")
                .and_then(|value| value.parse::<u64>().ok())
            {
                let percent = 20
                    + (processed_us / 1000)
                        .saturating_mul(70)
                        .checked_div(output_duration_ms.max(1))
                        .unwrap_or(0)
                        .min(70) as u8;
                report(progress, "renderingVideoEdit", percent)?;
            }
            Ok(())
        },
        |_| Ok(()),
    )?;
    if !run.status.success() {
        return Err(subprocess::diagnostic_error(
            &run.diagnostics,
            "FFmpeg could not render the video edit",
        ));
    }
    report(progress, "savingResource", 94)?;
    video_output::inspect(output.retain())
}

fn source_has_audio(ffmpeg: &PathBuf, source: &str) -> bool {
    Command::new(ffmpeg)
        .args([
            "-hide_banner",
            "-loglevel",
            "error",
            "-i",
            source,
            "-map",
            "0:a:0",
            "-t",
            "0.01",
            "-f",
            "null",
            "-",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}
