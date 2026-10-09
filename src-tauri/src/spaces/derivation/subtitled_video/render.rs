use std::{
    fs,
    path::Path,
    process::{Command, Stdio},
};

use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::persistence::VideoSubtitleSource;
use super::{progress, SubtitledVideoProgress, CANCELLED};
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
    system_font::SYSTEM_SANS_FAMILY,
};

pub fn render(
    app: &tauri::AppHandle,
    derivation_id: &str,
    source: VideoSubtitleSource,
    channel: &Channel<SubtitledVideoProgress>,
    cancellation: &CancellationToken,
) -> Result<RenderedVideo, String> {
    progress(channel, "preparingSubtitles", 62)?;
    let directory = asset_directory(app)?;
    let output = ScratchFile::new(directory.join(format!("derived-{}.mp4", Uuid::new_v4())));
    burn_in_subtitles(
        app,
        derivation_id,
        &directory,
        &source,
        output.path(),
        channel,
        cancellation,
    )?;
    progress(channel, "savingResource", 94)?;
    let rendered = video_output::inspect(output.path().to_path_buf())?;
    output.retain();
    Ok(rendered)
}

fn burn_in_subtitles(
    app: &tauri::AppHandle,
    derivation_id: &str,
    directory: &Path,
    source: &VideoSubtitleSource,
    output: &Path,
    channel: &Channel<SubtitledVideoProgress>,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let subtitle = ScratchFile::new(directory.join(format!("subtitle-{derivation_id}.srt")));
    fs::write(subtitle.path(), &source.srt).map_err(|error| error.to_string())?;
    run_ffmpeg(
        app,
        &source.path,
        subtitle.path(),
        output,
        channel,
        cancellation,
    )
}

fn run_ffmpeg(
    app: &tauri::AppHandle,
    source: &str,
    subtitle: &Path,
    output: &Path,
    channel: &Channel<SubtitledVideoProgress>,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let ffmpeg =
        tools::resolve_ffmpeg(app)?.ok_or("FFmpeg is required to create a subtitled video")?;
    progress(channel, "renderingSubtitles", 68)?;
    let subtitle_name = subtitle
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("Subtitle workspace path is invalid")?;
    let child = Command::new(ffmpeg)
        .args(ffmpeg_arguments(source, subtitle_name)?)
        .arg(output)
        .current_dir(
            subtitle
                .parent()
                .ok_or("Subtitle workspace is unavailable")?,
        )
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to launch FFmpeg: {error}"))?;
    let run = subprocess::wait_with_stderr(child, cancellation, CANCELLED, |_| Ok(()))?;
    if run.status.success() {
        Ok(())
    } else {
        Err(subprocess::diagnostic_error(
            &run.diagnostics,
            "FFmpeg could not create the subtitled video",
        ))
    }
}

pub(super) fn ffmpeg_arguments(source: &str, subtitle_name: &str) -> Result<Vec<String>, String> {
    let filter = format!(
        "subtitles={subtitle_name}:force_style='FontName={SYSTEM_SANS_FAMILY},Alignment=2,MarginL=36,MarginR=36,MarginV=28,FontSize=14,Outline=1.5,Shadow=0,WrapStyle=0'"
    );
    Ok(vec![
        "-hide_banner".into(),
        "-loglevel".into(),
        "error".into(),
        "-y".into(),
        "-i".into(),
        source.into(),
        "-vf".into(),
        filter,
        "-map".into(),
        "0:v:0".into(),
        "-map".into(),
        "0:a?".into(),
        "-c:v".into(),
        video_output::platform_h264_encoder()?.into(),
        "-b:v".into(),
        "8M".into(),
        "-maxrate".into(),
        "12M".into(),
        "-bufsize".into(),
        "16M".into(),
        "-pix_fmt".into(),
        "yuv420p".into(),
        "-c:a".into(),
        "copy".into(),
        "-movflags".into(),
        "+faststart".into(),
    ])
}
