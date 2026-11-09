//! Program loudness references and the mastering pass of the video dubbing mix.
//!
//! Every bus is first normalized statically to its element reference — dubbed speech anchors the
//! program at about −17 LUFS with beds 6 dB underneath, the conventional dialogue/music balance
//! for delivered video. The summed premaster is then mastered to the streaming delivery standard:
//! −16 LUFS integrated with −1.5 dB true-peak headroom for lossy encoding. Both stages use
//! measured static gains (see [`super::super::loudness`]) so the mix stays sample-exact.

use std::{path::Path, process::Command};

use tokio_util::sync::CancellationToken;

use super::super::loudness::LoudnessTarget;

/// The synthesized dialogue bus: the program anchor every other element is balanced against.
pub(super) const DIALOGUE_REFERENCE: LoudnessTarget = LoudnessTarget {
    loudness_lufs: -17.0,
    true_peak_db: -2.0,
};

/// Music/sfx beds and the preserved original audio, sitting under the dialogue.
pub(super) const BACKGROUND_REFERENCE: LoudnessTarget = LoudnessTarget {
    loudness_lufs: -23.0,
    true_peak_db: -2.0,
};

/// The delivered program: streaming-platform integrated loudness with encode headroom.
pub(super) const PROGRAM_MASTER: LoudnessTarget = LoudnessTarget {
    loudness_lufs: -16.0,
    true_peak_db: -1.5,
};

/// How far the bed dips under synthesized dialogue. From the −23 LUFS reference this puts speech
/// about 11 dB above the bed while it speaks — the conventional broadcast duck depth.
const DUCK_DEPTH_DB: f64 = 5.0;

/// The bed is already down when the first word starts; 120 ms of lookahead is inaudible as a
/// swell but removes every trace of the duck arriving late.
pub(super) const DUCK_ATTACK_SAMPLES: u64 = 5_760;

/// A 400 ms recovery is slow enough that the bed returns without audible pumping.
pub(super) const DUCK_RELEASE_SAMPLES: u64 = 16_800;

/// The linear duck-envelope value under dialogue, derived from [`DUCK_DEPTH_DB`].
pub(super) fn duck_level() -> f32 {
    10f32.powf(-(DUCK_DEPTH_DB as f32) / 20.0)
}

/// Applies the measured program gain and the delivery true-peak ceiling to the premaster. The
/// limiter only catches inter-sample peaks the static gain could not know about; the loudness of
/// the output is carried by the gain alone, so the master never pumps.
pub(super) fn master(
    ffmpeg: &Path,
    premaster: &Path,
    output: &Path,
    gain_db: f32,
    sample_count: u64,
    channels: u16,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let limit = 10f64.powf(PROGRAM_MASTER.true_peak_db / 20.0);
    let layout = if channels == 2 { "stereo" } else { "mono" };
    let filter = format!(
        "[0:a:0]volume={gain_db:.6}dB,alimiter=limit={limit:.9}:level=false:latency=true,atrim=end_sample={sample_count},aformat=sample_fmts=flt:channel_layouts={layout}[out]"
    );
    crate::media::ffmpeg::run(
        Command::new(ffmpeg)
            .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-n", "-i"])
            .arg(premaster)
            .arg("-filter_complex")
            .arg(filter)
            .args(["-map", "[out]", "-c:a", "pcm_f32le", "-f", "wav"])
            .arg(output),
        cancellation,
        "FFmpeg could not master video dubbing audio",
    )?;
    if super::policy::wav_sample_count(output, channels)? != sample_count {
        return Err("Mastered video dubbing audio has an invalid duration".into());
    }
    Ok(())
}
