use std::{path::Path, process::Command};
use tokio_util::sync::CancellationToken;

use super::SAMPLE_RATE;

/// How the source picture enters the published MP4.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Picture {
    /// The stream is copied untouched: MP4 carries it and players read it as tagged.
    Copy,
    /// The stream is copied under the sample-entry tag every player reads. FFmpeg writes HEVC
    /// as `hev1` by default, which QuickTime, Safari and iOS refuse; `hvc1` plays everywhere.
    CopyTagged(&'static str),
    /// MP4 cannot carry the codec (ProRes, DNxHD, …): the picture is encoded to H.264.
    Encode,
}

impl Picture {
    pub(super) fn of(codec: &str) -> Self {
        match codec {
            "h264" | "av1" | "vp9" | "mpeg4" => Self::Copy,
            "hevc" => Self::CopyTagged("hvc1"),
            _ => Self::Encode,
        }
    }

    fn arguments(&self) -> Result<Vec<&'static str>, String> {
        Ok(match self {
            Self::Copy => vec!["-c:v", "copy"],
            Self::CopyTagged(tag) => vec!["-c:v", "copy", "-tag:v", tag],
            Self::Encode => vec![
                "-c:v",
                crate::spaces::derivation::platform_h264_encoder()?,
                "-b:v",
                "8M",
                "-maxrate",
                "12M",
                "-bufsize",
                "16M",
                "-pix_fmt",
                "yuv420p",
            ],
        })
    }
}

/// Publishes the dubbed video. The picture is copied whenever MP4 carries it ([`Picture`]).
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
    let picture = Picture::of(&crate::media::ffmpeg::video_codec(
        ffmpeg,
        video,
        cancellation,
    )?);
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
        ])
        .args(picture.arguments()?)
        .args([
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
