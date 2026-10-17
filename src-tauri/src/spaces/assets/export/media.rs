use std::{path::Path, process::Command};

use tokio_util::sync::CancellationToken;

use crate::{
    file_inspection::verify_content_revision, foundation::scratch_file::ScratchFile,
    local_resources::tools, media::ffmpeg,
};

/// One encoding the bundled FFmpeg is qualified to produce. The arguments are
/// the whole definition of the target, so adding a format adds a row.
#[derive(Debug, Eq, PartialEq)]
pub(super) struct Target {
    pub extension: &'static str,
    arguments: &'static [&'static str],
}

const AUDIO: &[Target] = &[
    Target {
        extension: "wav",
        arguments: &["-vn", "-c:a", "pcm_s16le"],
    },
    Target {
        extension: "mp3",
        arguments: &["-vn", "-c:a", "libmp3lame", "-q:a", "2"],
    },
    Target {
        extension: "m4a",
        arguments: &["-vn", "-c:a", "aac", "-b:a", "192k"],
    },
    Target {
        extension: "flac",
        arguments: &["-vn", "-c:a", "flac"],
    },
];

const VIDEO: &[Target] = &[
    Target {
        extension: "mp4",
        arguments: VIDEO_ARGUMENTS,
    },
    Target {
        extension: "mov",
        arguments: VIDEO_ARGUMENTS,
    },
];

/// H.264 and AAC, the pair every player and editor admits. `faststart` costs one
/// rewrite of the index and makes the result playable while it downloads.
const VIDEO_ARGUMENTS: &[&str] = &[
    "-c:v",
    "libx264",
    "-preset",
    "medium",
    "-crf",
    "20",
    "-pix_fmt",
    "yuv420p",
    "-c:a",
    "aac",
    "-b:a",
    "192k",
    "-movflags",
    "+faststart",
];

pub(super) fn targets(kind: &str) -> &'static [Target] {
    match kind {
        "audio" => AUDIO,
        "video" => VIDEO,
        _ => &[],
    }
}

/// Encodes into the destination the user confirmed. A failed or cancelled run
/// leaves no partial file behind, and the source is checked again afterwards so
/// a recording that changed mid-export cannot be delivered as the current one.
pub(super) fn transcode(
    app: &tauri::AppHandle,
    source: &Path,
    source_revision: &str,
    target: &'static Target,
    destination: &Path,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let executable = tools::resolve_ffmpeg(app)?.ok_or("Bundled FFmpeg is unavailable")?;
    crate::spaces::assets::authorize_destination(app, destination)?;
    verify_content_revision(source, source_revision)?;
    let file = ScratchFile::new(destination.to_owned());
    let mut command = Command::new(executable);
    command
        .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y", "-i"])
        .arg(source)
        .args(target.arguments)
        .arg(destination);
    ffmpeg::run(
        &mut command,
        cancellation,
        "FFmpeg could not convert this resource",
    )?;
    verify_content_revision(source, source_revision)?;
    file.retain();
    Ok(())
}
