use std::{path::Path, process::Command};
use tokio_util::sync::CancellationToken;

use super::SAMPLE_RATE;

/// Publishes the dubbed video.
///
/// Every published track is as long as the verified picture, stated here rather than negotiated:
/// each one is padded and then trimmed to the sample count the picture was measured at. The
/// alternative - padding without a bound and letting `-shortest` end the file - makes the published
/// length depend on the length of the source's own audio track, which is unrelated to the picture.
pub(super) fn mux_video(
    ffmpeg: &Path,
    video: &Path,
    audio: &Path,
    output: &Path,
    cancellation: &CancellationToken,
    retain_original: bool,
    video_samples: u64,
) -> Result<(), String> {
    let dubbed = format!("[1:a:0]apad,atrim=end_sample={video_samples}[audio]");
    let filter = if retain_original {
        format!("{dubbed};[0:a:0]aresample={SAMPLE_RATE},apad,atrim=end_sample={video_samples}[original]")
    } else {
        dubbed
    };
    let mut command = Command::new(ffmpeg);
    command
        .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y", "-i"])
        .arg(video)
        .arg("-i")
        .arg(audio)
        .args([
            "-filter_complex",
            &filter,
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
