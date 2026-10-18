use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use tokio_util::sync::CancellationToken;

use super::{mixing::TimedRender, SAMPLE_RATE};
use crate::spaces::speech::publication::hash_file;

/// How much faster a line may be played to fit the time the subtitles gave it.
///
/// Compressing speech by up to a quarter stays natural enough for dubbing. A line that needs more
/// than that keeps speaking into the gap that follows instead of being cut short, and the voice
/// lanes carry the overlap if the next line starts first. Nothing is ever slowed down to fill a
/// window: speech that is shorter than its window simply leaves silence behind it.
const MAX_COMPRESSION: f64 = 1.25;

/// Where a line is heard and for how long, after fitting it to the time it was given.
pub(super) struct Placement {
    pub(super) start_sample: u64,
    pub(super) sample_count: u64,
    tempo: f64,
}

impl Placement {
    pub(super) fn end_sample(&self) -> u64 {
        self.start_sample + self.sample_count
    }
}

/// Decides how every line is heard. The subtitle window is a target, not a contract: the room a
/// line really has reaches to the start of the next one, and no line is allowed past the video.
pub(super) fn plan(timed: &[TimedRender], video_samples: u64) -> Result<Vec<Placement>, String> {
    let mut placements = Vec::with_capacity(timed.len());
    for (index, item) in timed.iter().enumerate() {
        let room = timed
            .get(index + 1)
            .map_or(video_samples, |next| next.start_sample)
            .max(item.start_sample + 1)
            .min(video_samples);
        placements.push(placement(item, room, video_samples)?);
    }
    Ok(placements)
}

fn placement(item: &TimedRender, room: u64, video_samples: u64) -> Result<Placement, String> {
    let natural = natural_samples(item)?;
    let limit = video_samples
        .checked_sub(item.start_sample)
        .filter(|value| *value > 0)
        .ok_or("Video target window starts after the source ends")?;
    let preferred = item.window_samples.min(room - item.start_sample);
    let mut tempo = if preferred > 0 && natural > preferred {
        (natural as f64 / preferred as f64).min(MAX_COMPRESSION)
    } else {
        1.0
    };
    let sample_count = scaled(natural, tempo).clamp(1, limit);
    if sample_count == limit && natural > limit {
        tempo = natural as f64 / limit as f64;
    }
    Ok(Placement {
        start_sample: item.start_sample,
        sample_count,
        tempo,
    })
}

/// How long a line is at the rate it was spoken, in the sample rate the mix works in.
fn natural_samples(item: &TimedRender) -> Result<u64, String> {
    if item.render.sample_rate == 0 {
        return Err("Speech segment has an invalid sample rate".into());
    }
    let samples = u128::from(item.render.sample_count) * u128::from(SAMPLE_RATE)
        / u128::from(item.render.sample_rate);
    u64::try_from(samples).map_err(|_| "Speech segment duration overflow".into())
}

fn scaled(natural: u64, tempo: f64) -> u64 {
    (natural as f64 / tempo).ceil() as u64
}

/// Renders one line as it will be heard: at its fitted rate, for exactly its placed length.
pub(super) fn render(
    ffmpeg: &Path,
    work: &Path,
    index: usize,
    item: &TimedRender,
    placement: &Placement,
    cancellation: &CancellationToken,
) -> Result<PathBuf, String> {
    let source = Path::new(&item.render.path);
    if hash_file(source)? != item.render.sha256 {
        return Err("Video speech segment digest differs from its render metadata".into());
    }
    let path = work.join(format!("fit-{index}.wav"));
    let tempo = if placement.tempo == 1.0 {
        String::new()
    } else {
        format!("atempo={:.9},", placement.tempo)
    };
    let samples = placement.sample_count;
    let filter = format!(
        "aresample={SAMPLE_RATE},{tempo}apad,atrim=end_sample={samples},aformat=sample_fmts=flt:channel_layouts=mono"
    );
    crate::media::ffmpeg::run(
        Command::new(ffmpeg)
            .args(["-hide_banner", "-loglevel", "error", "-nostdin", "-n", "-i"])
            .arg(source)
            .args([
                "-map",
                "0:a:0",
                "-af",
                &filter,
                "-c:a",
                "pcm_f32le",
                "-f",
                "wav",
            ])
            .arg(&path),
        cancellation,
        "FFmpeg could not fit speech to its video window",
    )?;
    if super::policy::wav_sample_count(&path, 1)? != samples {
        return Err(cleanup_failure(
            "Fitted speech does not match its placed sample count".into(),
            &path,
        ));
    }
    Ok(path)
}

fn cleanup_failure(mut cause: String, path: &Path) -> String {
    if let Err(error) = fs::remove_file(path) {
        if error.kind() != std::io::ErrorKind::NotFound {
            cause.push_str(&format!("; failed to remove {}: {error}", path.display()));
        }
    }
    cause
}

#[cfg(test)]
#[path = "tests/fitting.rs"]
mod tests;
