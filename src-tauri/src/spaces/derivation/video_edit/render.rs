use std::{
    fs,
    process::{Command, Stdio},
};

use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::{ass, filter_graph, report, start::VideoEditSource, VideoEditProgress, CANCELLED};
use crate::{
    foundation::scratch_file::ScratchFile,
    foundation::subprocess,
    local_resources::tools,
    spaces::{
        assets::asset_directory,
        derivation::video_output::{self, RenderedVideo},
    },
};

pub(super) fn render(
    app: &tauri::AppHandle,
    source: VideoEditSource,
    progress: &Channel<VideoEditProgress>,
    cancellation: &CancellationToken,
) -> Result<RenderedVideo, String> {
    report(progress, "preparingVideoEdit", 8)?;
    let ffmpeg = tools::resolve_ffmpeg(app)?.ok_or("FFmpeg is required to render a video edit")?;
    let directory = asset_directory(app)?;
    let output = ScratchFile::new(directory.join(format!("edited-{}.mp4", Uuid::new_v4())));
    let project = &source.project;
    let output_duration_ms = super::timing::duration_ms(project);
    let (input_args, has_audio) = super::render_inputs::prepare(&ffmpeg, &source, cancellation)?;
    let subtitle = if project.text_track.items.is_empty() {
        None
    } else {
        let file = ScratchFile::new(directory.join(format!("video-edit-{}.ass", Uuid::new_v4())));
        fs::write(file.path(), ass::render(project)).map_err(|error| error.to_string())?;
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
    let filter = filter_graph::build(project, &has_audio, subtitle_name)?;
    report(progress, "renderingVideoEdit", 20)?;
    let mut args = [
        "-nostdin",
        "-hide_banner",
        "-loglevel",
        "error",
        "-filter_complex_threads",
        "2",
        "-stats_period",
        "0.5",
        "-progress",
        "pipe:1",
        "-y",
    ]
    .map(str::to_owned)
    .to_vec();
    args.extend(input_args);
    args.extend(
        [
            "-filter_complex",
            &filter,
            "-map",
            "[outv]",
            "-map",
            "[outa]",
            "-c:a",
            "aac",
            "-b:a",
            "192k",
        ]
        .map(str::to_owned),
    );
    let encoder = video_output::platform_h264_encoder()?;
    args.extend(
        [
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
        ]
        .map(str::to_owned),
    );
    args.extend(["-t".into(), super::timing::seconds(output_duration_ms)]);
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
    video_output::inspect(app, output.retain(), cancellation)
}
