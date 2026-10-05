use std::{path::Path, process::Command};
use tokio_util::sync::CancellationToken;

pub(super) fn mux_video(
    ffmpeg: &Path,
    video: &Path,
    audio: &Path,
    output: &Path,
    cancellation: &CancellationToken,
    retain_original: bool,
) -> Result<(), String> {
    let filter = if retain_original {
        "[1:a:0]apad[audio];[0:a:0]apad[original]"
    } else {
        "[1:a:0]apad[audio]"
    };
    let mut command = Command::new(ffmpeg);
    command
        .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y", "-i"])
        .arg(video)
        .arg("-i")
        .arg(audio)
        .args([
            "-filter_complex",
            filter,
            "-map",
            "0:v:0",
            "-map",
            "[audio]",
            "-disposition:a:0",
            "default",
            "-metadata:s:a:0",
            "title=Dubbed",
            "-c:v",
            "copy",
            "-c:a",
            "aac",
            "-ac",
            "2",
            "-b:a",
            "192k",
            "-shortest",
            "-movflags",
            "+faststart",
            "-f",
            "mp4",
        ]);
    if retain_original {
        command.args([
            "-map",
            "[original]",
            "-disposition:a:1",
            "0",
            "-metadata:s:a:1",
            "title=Original",
        ]);
    }
    command.arg(output);
    crate::media::ffmpeg::run(
        &mut command,
        cancellation,
        "FFmpeg could not publish the dubbed video",
    )
}
