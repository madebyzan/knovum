use std::path::{Path, PathBuf};

use tokio_util::sync::CancellationToken;

use super::super::{
    contracts::SpeechSegmentRenderOutput,
    loudness::{self, LoudnessInput, LoudnessTarget},
    video_config::VideoDubbingOutputConfig,
};
use super::{
    final_mix::{mix_final, FinalMixSpec, MixOutput},
    fitting, mastering, SAMPLE_RATE,
};

/// A line and the time the subtitles gave it. How long it is actually heard for is decided when
/// the mix is planned, because the window is a target rather than a contract.
pub(in crate::spaces::speech) struct TimedRender {
    pub(super) render: SpeechSegmentRenderOutput,
    pub(super) start_sample: u64,
    pub(super) window_samples: u64,
}

pub(in crate::spaces::speech) fn timed_renders(
    renders: &[SpeechSegmentRenderOutput],
    windows: &[(Option<i64>, Option<i64>)],
    video_samples: u64,
) -> Result<Vec<TimedRender>, String> {
    let mut timed = renders
        .iter()
        .cloned()
        .zip(windows)
        .map(|(render, window)| {
            let (Some(start), Some(end)) = *window else {
                return Err("Video dubbing segment has no target time window".into());
            };
            let start = samples_at(start, "Video target start")?;
            if start >= video_samples {
                return Err("Video target window starts after the source ends".into());
            }
            let end = samples_at(end, "Video target end")?.min(video_samples);
            Ok(TimedRender {
                render,
                start_sample: start,
                window_samples: end.saturating_sub(start),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    timed.sort_by_key(|item| (item.start_sample, item.window_samples));
    Ok(timed)
}

fn samples_at(milliseconds: i64, label: &str) -> Result<u64, String> {
    Ok(u64::try_from(milliseconds)
        .map_err(|_| format!("{label} is invalid"))?
        .checked_mul(u64::from(SAMPLE_RATE))
        .ok_or_else(|| format!("{label} overflow"))?
        / 1_000)
}

pub(in crate::spaces::speech) fn build_mix(
    ffmpeg: &Path,
    work: &Path,
    timed: &[TimedRender],
    video_samples: u64,
    background: Option<&Path>,
    config: &VideoDubbingOutputConfig,
    cancellation: &CancellationToken,
) -> Result<PathBuf, String> {
    build_artifact(
        MixRequest {
            ffmpeg,
            work,
            timed,
            video_samples,
            background,
            config,
            cancellation,
        },
        MixOutput::Combined,
    )
}

pub(in crate::spaces::speech) struct MixRequest<'a> {
    pub ffmpeg: &'a Path,
    pub work: &'a Path,
    pub timed: &'a [TimedRender],
    pub video_samples: u64,
    pub background: Option<&'a Path>,
    pub config: &'a VideoDubbingOutputConfig,
    pub cancellation: &'a CancellationToken,
}

pub(in crate::spaces::speech) fn build_stem(
    request: MixRequest<'_>,
    kind: MixOutput,
) -> Result<PathBuf, String> {
    build_artifact(request, kind)
}

fn build_artifact(request: MixRequest<'_>, kind: MixOutput) -> Result<PathBuf, String> {
    let MixRequest {
        ffmpeg,
        work,
        timed,
        video_samples,
        background,
        config,
        cancellation,
    } = request;
    let timed = if kind == MixOutput::Background {
        &[]
    } else {
        timed
    };
    let background = if kind == MixOutput::Dialogue {
        None
    } else {
        background
    };
    if kind == MixOutput::Background && background.is_none() {
        return Err("This dubbing revision has no background audio".into());
    }
    let placements = fitting::plan(timed, video_samples)?;
    let mut fitted = Vec::with_capacity(placements.len());
    for (ordinal, placement) in placements.iter().enumerate() {
        if cancellation.is_cancelled() {
            return Err("cancelled".into());
        }
        fitted.push(fitting::render(
            ffmpeg,
            work,
            ordinal,
            &timed[placement.index],
            placement,
            cancellation,
        )?);
    }
    let dialogue = if placements.is_empty() {
        None
    } else {
        let path = work.join("dialogue.wav");
        super::dialogue_track::write(&path, &placements, &fitted, video_samples, cancellation)?;
        if super::policy::wav_sample_count(&path, 1)? != video_samples {
            return Err("Video dialogue track has an invalid duration".into());
        }
        Some(path)
    };
    // The swap between separated bed and preserved original belongs to every artifact that
    // carries the bed; ducking belongs to the delivered program only, never to clean stems.
    let swap = config.preserve_original_outside_dialogue && background.is_some();
    let envelope = if swap {
        let path = work.join("dialogue-envelope.wav");
        super::envelope::write_swap(
            &path,
            video_samples,
            &config.source_dialogue_intervals,
            &placements,
            cancellation,
        )?;
        Some(path)
    } else {
        None
    };
    let duck = if kind == MixOutput::Combined && background.is_some() && dialogue.is_some() {
        let path = work.join("duck-envelope.wav");
        super::envelope::write_duck(&path, video_samples, &placements, cancellation)?;
        Some(path)
    } else {
        None
    };
    let gains = bus_gains(
        ffmpeg,
        BusSources {
            dialogue: dialogue.as_deref(),
            background,
            swap_video: swap.then_some(Path::new(&config.video.asset_path)),
        },
        // User trims belong to the delivered program; stems carry reference levels only.
        if kind == MixOutput::Combined {
            (config.voice_gain_db, config.background_gain_db)
        } else {
            (0.0, 0.0)
        },
        cancellation,
    )?;
    let spec = FinalMixSpec {
        sample_count: video_samples,
        dialogue_gain_db: gains.dialogue_db,
        background_gain_db: gains.background_db,
        original_gain_db: gains.original_db,
        duck: duck.as_deref(),
        original: swap.then_some(Path::new(&config.video.asset_path)),
        envelope: envelope.as_deref(),
        output_kind: kind,
    };
    let mixed = work.join("mixed.wav");
    if kind != MixOutput::Combined {
        mix_final(
            ffmpeg,
            dialogue.as_deref(),
            background,
            &mixed,
            spec,
            cancellation,
        )?;
        return Ok(mixed);
    }
    let premaster = work.join("premaster.wav");
    mix_final(
        ffmpeg,
        dialogue.as_deref(),
        background,
        &premaster,
        spec,
        cancellation,
    )?;
    let measured = loudness::measure(
        ffmpeg,
        LoudnessInput::File(&premaster),
        SAMPLE_RATE,
        &mastering::PROGRAM_MASTER,
        cancellation,
    )?;
    mastering::master(
        ffmpeg,
        &premaster,
        &mixed,
        loudness::static_gain_db(&measured, &mastering::PROGRAM_MASTER) as f32,
        video_samples,
        if background.is_some() { 2 } else { 1 },
        cancellation,
    )?;
    Ok(mixed)
}

struct BusSources<'a> {
    dialogue: Option<&'a Path>,
    background: Option<&'a Path>,
    swap_video: Option<&'a Path>,
}

struct BusGains {
    dialogue_db: f32,
    background_db: f32,
    original_db: f32,
}

/// Normalizes every present bus statically to its element reference and folds the user trims in,
/// so the mix graph only ever applies fixed decibel values.
fn bus_gains(
    ffmpeg: &Path,
    sources: BusSources<'_>,
    trims: (f32, f32),
    cancellation: &CancellationToken,
) -> Result<BusGains, String> {
    let (voice_trim, background_trim) = trims;
    let dialogue_db = match sources.dialogue {
        Some(path) => {
            reference_gain_db(ffmpeg, path, &mastering::DIALOGUE_REFERENCE, cancellation)?
                + voice_trim
        }
        None => 0.0,
    };
    let background_db = match sources.background {
        Some(path) => {
            reference_gain_db(ffmpeg, path, &mastering::BACKGROUND_REFERENCE, cancellation)?
                + background_trim
        }
        None => 0.0,
    };
    // The preserved original is normalized to the same reference as the bed, which makes the
    // swap between them loudness-neutral, then carries the same user trim.
    let original_db = match sources.swap_video {
        Some(path) => {
            reference_gain_db(ffmpeg, path, &mastering::BACKGROUND_REFERENCE, cancellation)?
                + background_trim
        }
        None => 0.0,
    };
    Ok(BusGains {
        dialogue_db,
        background_db,
        original_db,
    })
}

fn reference_gain_db(
    ffmpeg: &Path,
    source: &Path,
    reference: &LoudnessTarget,
    cancellation: &CancellationToken,
) -> Result<f32, String> {
    let measured = loudness::measure(
        ffmpeg,
        LoudnessInput::File(source),
        SAMPLE_RATE,
        reference,
        cancellation,
    )?;
    Ok(loudness::static_gain_db(&measured, reference) as f32)
}
