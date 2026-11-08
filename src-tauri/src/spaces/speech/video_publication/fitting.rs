use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use tokio_util::sync::CancellationToken;

use super::{mixing::TimedRender, SAMPLE_RATE};
use crate::spaces::speech::publication::hash_file;

/// Minimum tempo (slowing down speech) to match a slower speaker in the video.
/// Slowing down to 0.70x (1.43x duration) sounds natural and measured without distortion.
pub const MIN_TEMPO: f64 = 0.70;

/// Maximum tempo (speeding up speech) to fit a fast speaker or compact dialogue window.
/// Speeding up to 1.50x (0.67x duration) preserves speech clarity and intonation.
pub const MAX_TEMPO: f64 = 1.50;

/// The fade every placed line ends with: long enough to avoid a click, short enough to be
/// inaudible as a fade.
const FADE_SAMPLES: u64 = 480;

/// Where a line is heard and for how long, after fitting it to the time it was given.
pub(super) struct Placement {
    /// The line this placement speaks, as indexed into the timed renders it was planned from.
    pub(super) index: usize,
    pub(super) start_sample: u64,
    pub(super) sample_count: u64,
    tempo: f64,
}

impl Placement {
    pub(super) fn end_sample(&self) -> u64 {
        self.start_sample + self.sample_count
    }
}

/// Decides how every line is heard to maintain synchronization with the video picture.
/// Dialogue is spoken in script order: each line anchors to the timestamp the subtitles gave it,
/// adapting its pacing bidirectionally within natural speech limits so that speech matches visual
/// delivery without accumulating drift across scenes.
pub(super) fn plan(timed: &[TimedRender], video_samples: u64) -> Result<Vec<Placement>, String> {
    let mut placements = Vec::with_capacity(timed.len());
    let mut cursor = 0;
    for (index, item) in timed.iter().enumerate() {
        let start = item.start_sample.max(cursor);
        let picture = video_samples.saturating_sub(start);
        if picture == 0 {
            // The picture is over, so this line cannot be heard.
            continue;
        }
        let next = timed
            .get(index + 1)
            .map_or(video_samples, |next| next.start_sample);
        let placement = placement(
            index,
            item,
            Room {
                start,
                next,
                picture,
            },
        )?;
        cursor = placement.end_sample();
        placements.push(placement);
    }
    Ok(placements)
}

/// The time a line really has: when it can start speaking, when the next line is due, and how
/// much picture is left.
struct Room {
    start: u64,
    next: u64,
    picture: u64,
}

fn placement(index: usize, item: &TimedRender, room: Room) -> Result<Placement, String> {
    let natural = natural_samples(item)?;
    let delay_samples = room.start.saturating_sub(item.start_sample);
    let available_window = item.window_samples.saturating_sub(delay_samples);
    let max_room = room
        .next
        .saturating_sub(room.start)
        .min(room.picture)
        .max(1);
    let target = available_window.min(max_room).max(1);

    let raw_tempo = natural as f64 / target as f64;
    let (tempo, sample_count) = if (raw_tempo - 1.0).abs() < 0.02 {
        (1.0, natural.min(max_room))
    } else if raw_tempo > 1.0 {
        // Speech is longer than the dialogue window: speed up to match visual delivery.
        let t = raw_tempo.min(MAX_TEMPO);
        let count = scaled(natural, t).min(max_room);
        (t, count)
    } else {
        // Speech is shorter than the dialogue window: slow down to match visual delivery.
        let t = raw_tempo.max(MIN_TEMPO);
        let count = scaled(natural, t).min(target).max(1);
        (t, count)
    };

    Ok(Placement {
        index,
        start_sample: room.start,
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
    // Every placed line ends with a short fade, so a line the picture cuts short never clicks. A
    // line that ends in its own silence fades that silence and is unchanged.
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
