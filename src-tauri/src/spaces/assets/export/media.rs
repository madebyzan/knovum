use std::{path::Path, process::Command};

use tokio_util::sync::CancellationToken;

use crate::{
    file_inspection::verify_content_revision, foundation::scratch_file::ScratchFile,
    local_resources::tools, media::ffmpeg,
};

/// One encoding the bundled FFmpeg is qualified to produce. A row is the whole definition of a
/// target: what to encode with, and what its container asks for on top of that.
#[derive(Debug, Eq, PartialEq)]
pub(super) struct Target {
    pub extension: &'static str,
    codecs: &'static [&'static str],
    container: &'static [&'static str],
}

/// Every encoding the bundled FFmpeg is qualified to write sound in: lossless and large,
/// the one every device plays, the one every Apple tool prefers, and lossless and compressed.
static AUDIO: [Target; 4] = [
    Target {
        extension: "wav",
        codecs: &["-vn", "-c:a", "pcm_s16le"],
        container: &[],
    },
    Target {
        extension: "mp3",
        codecs: &["-vn", "-c:a", "libmp3lame", "-q:a", "2"],
        container: &[],
    },
    Target {
        extension: "m4a",
        codecs: &["-vn", "-c:a", "aac", "-b:a", "192k"],
        container: &[],
    },
    Target {
        extension: "flac",
        codecs: &["-vn", "-c:a", "flac"],
        container: &[],
    },
];

/// The containers that carry the one qualified codec pair. The codecs are the whole of what a
/// player admits, so a container is a choice of wrapper rather than of quality.
static VIDEO: [Target; 3] = [
    Target {
        extension: "mp4",
        codecs: VIDEO_CODECS,
        container: FASTSTART,
    },
    Target {
        extension: "mov",
        codecs: VIDEO_CODECS,
        container: FASTSTART,
    },
    Target {
        extension: "mkv",
        codecs: VIDEO_CODECS,
        container: &[],
    },
];

/// H.264 and AAC, the pair every player and editor admits.
const VIDEO_CODECS: &[&str] = &[
    "-c:v", "libx264", "-preset", "medium", "-crf", "20", "-pix_fmt", "yuv420p", "-c:a", "aac",
    "-b:a", "192k",
];

/// `faststart` costs one rewrite of the index and makes the result playable while it downloads.
/// It is an MP4-family option, so Matroska does not ask for it.
const FASTSTART: &[&str] = &["-movflags", "+faststart"];

pub(super) fn targets(kind: &str) -> Vec<&'static Target> {
    match kind {
        "audio" => AUDIO.iter().collect(),
        // A video carries sound as well as pictures, so taking the sound out of it is one of its
        // encodings rather than a separate feature. Every audio target drops the video stream.
        "video" => VIDEO.iter().chain(AUDIO.iter()).collect(),
        _ => vec![],
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
        .args(target.codecs)
        .args(target.container)
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
