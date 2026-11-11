use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use tokio_util::sync::CancellationToken;

use super::{mixing::TimedRender, SAMPLE_RATE};
use crate::spaces::speech::publication::hash_file;

/// How much faster a line may be played to land in its time.
///
/// Time-compressed speech keeps its natural pitch and stays fully intelligible to about 1.25×;
/// beyond that it is audibly hurried, and a line that still needs more room takes it from the
/// lines after it instead. Speech is never slowed down to fill time.
pub(super) const MAX_COMPRESSION: f64 = 1.25;

/// The fade every placed line ends with: long enough to avoid a click, short enough to be
/// inaudible as a fade.
const FADE_SAMPLES: u64 = 480;

/// Where a line is heard and for how long, after fitting it to the time it was given.
pub(super) struct Placement {
    /// The line this placement speaks, as indexed into the timed renders it was planned from.
    pub(super) index: usize,
    pub(super) start_sample: u64,
    pub(super) sample_count: u64,
    pub(super) tempo: f64,
}

impl Placement {
    pub(super) fn end_sample(&self) -> u64 {
        self.start_sample + self.sample_count
    }
}

/// Decides how every line is heard. Every line is heard whole — a word is never cut.
///
/// A line starts at its subtitle onset, or as soon as the line before it has finished. It aims to
/// end with its subtitle interval and is compressed up to [`MAX_COMPRESSION`] to do so; what is
/// still left over is spoken into the silence before the next line, at most that fast again, and
/// past the next onset only when the line cannot fit at all, delaying the next line by exactly
/// that much. The delay is absorbed by the next silence long enough to hold it. Dialogue that
/// cannot end before the picture does is refused before anything is rendered, naming how much
/// longer it is, because the only alternative is dialogue nobody hears.
pub(super) fn plan(timed: &[TimedRender], video_samples: u64) -> Result<Vec<Placement>, String> {
    let mut placements = Vec::with_capacity(timed.len());
    let mut cursor = 0;
    for (index, item) in timed.iter().enumerate() {
        let start = item.start_sample.max(cursor);
        let next = timed
            .get(index + 1)
            .map_or(video_samples, |next| next.start_sample.min(video_samples));
        let natural = natural_samples(item)?;
        // Its own interval, then the silence before the next line (for the last line, the rest
        // of the picture). A target already behind the start allows nothing and gives way.
        let window_end = (item.start_sample + item.window_samples).min(next);
        let tempo = [window_end, next]
            .into_iter()
            .map(|end| tempo_to_end_by(natural, start, end))
            .find(|tempo| *tempo <= MAX_COMPRESSION)
            .unwrap_or(MAX_COMPRESSION);
        let sample_count = scaled(natural, tempo);
        let end = start + sample_count;
        if end > video_samples {
            return Err(format!(
                "Dubbed dialogue runs {:.2} s past the end of the video; shorten the last lines",
                (end - video_samples) as f64 / f64::from(SAMPLE_RATE)
            ));
        }
        placements.push(Placement {
            index,
            start_sample: start,
            sample_count,
            tempo,
        });
        cursor = end;
    }
    Ok(placements)
}

/// The rate at which `natural` samples spoken from `start` end by `end`: never below the natural
/// rate, and infinite when there is no time at all.
fn tempo_to_end_by(natural: u64, start: u64, end: u64) -> f64 {
    match end.checked_sub(start) {
        Some(room) if room > 0 => (natural as f64 / room as f64).max(1.0),
        _ => f64::INFINITY,
    }
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
    (natural as f64 / tempo).round() as u64
}

/// Renders one line as it will be heard: at its fitted rate, for exactly its placed length.
pub(super) fn render(
    ffmpeg: &Path,
    work: &Path,
    ordinal: usize,
    item: &TimedRender,
    placement: &Placement,
    cancellation: &CancellationToken,
) -> Result<PathBuf, String> {
    let source = Path::new(&item.render.path);
    if hash_file(source)? != item.render.sha256 {
        return Err("Video speech segment digest differs from its render metadata".into());
    }
    let path = work.join(format!("fit-{ordinal}.wav"));
    let tempo = if (placement.tempo - 1.0).abs() < 0.001 {
        String::new()
    } else {
        format!("atempo={:.9},", placement.tempo)
    };
    let samples = placement.sample_count;
    // The time-stretched line is trimmed to its exact placed length, which can end a waveform
    // mid-cycle; a fade too short to hear as one keeps that edge from clicking.
    let fade_start = samples.saturating_sub(FADE_SAMPLES) as f64 / f64::from(SAMPLE_RATE);
    let fade = FADE_SAMPLES.min(samples) as f64 / f64::from(SAMPLE_RATE);
    let filter = format!(
        "aresample={SAMPLE_RATE},{tempo}apad,atrim=end_sample={samples},afade=t=out:st={fade_start:.9}:d={fade:.9},aformat=sample_fmts=flt:channel_layouts=mono"
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
                "-ar",
                &SAMPLE_RATE.to_string(),
                "-ac",
                "1",
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
