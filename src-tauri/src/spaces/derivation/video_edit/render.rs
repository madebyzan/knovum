use std::{
    fs,
    path::PathBuf,
    process::{Command, Stdio},
};

use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::{filter_graph, report, start::VideoEditSource, VideoEditProgress, CANCELLED};
use crate::{
    local_resources::tools,
    media::subprocess,
    spaces::{
        assets::asset_directory,
        derivation::{
            scratch_file::ScratchFile,
            video_output::{self, RenderedVideo},
        },
    },
};

pub fn render(
    app: &tauri::AppHandle,
    source: VideoEditSource,
    progress: &Channel<VideoEditProgress>,
    cancellation: &CancellationToken,
) -> Result<RenderedVideo, String> {
    report(progress, "preparingVideoEdit", 8)?;
    let ffmpeg = tools::resolve_tool(app, "KNOVUM_FFMPEG_PATH", tools::ffmpeg_name())
        .ok_or("FFmpeg is required to render a video edit")?;
    let directory = asset_directory(app)?;
    let output = ScratchFile::new(directory.join(format!("edited-{}.mp4", Uuid::new_v4())));
    let font = ScratchFile::new(directory.join(format!("video-edit-font-{}.otf", Uuid::new_v4())));
    let bundled_font = tools::resolve_shared_file(app, "fonts/NotoSansCJKsc-Regular.otf")
        .ok_or("Bundled video editor font is unavailable")?;
    fs::copy(bundled_font, font.path()).map_err(|error| error.to_string())?;
    let font_name = font
        .path()
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Video editor font path is invalid")?;
    let has_audio = source_has_audio(&ffmpeg, &source.path);
    let filter = filter_graph::build(&source.project, has_audio, font_name)?;
    report(progress, "renderingVideoEdit", 20)?;
    let mut args = vec![
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
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
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to launch FFmpeg: {error}"))?;
    let run = subprocess::wait_with_stderr(child, cancellation, CANCELLED, |_| Ok(()))?;
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
