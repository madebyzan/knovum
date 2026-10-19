use std::path::{Path, PathBuf};

use tokio_util::sync::CancellationToken;

use super::super::{contracts::SpeechSegmentRenderOutput, video_config::VideoDubbingOutputConfig};
use super::{
    final_mix::{mix_final, FinalMixSpec, MixOutput},
    fitting, SAMPLE_RATE,
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
    let envelope = if config.preserve_original_outside_dialogue && background.is_some() {
        let path = work.join("dialogue-envelope.wav");
        super::dialogue_envelope::write(
            &path,
            video_samples,
            &config.source_dialogue_intervals,
            cancellation,
        )?;
        Some(path)
    } else {
        None
    };
    let mixed = work.join("mixed.wav");
    mix_final(
        ffmpeg,
        dialogue.as_deref(),
        background,
        &mixed,
        FinalMixSpec {
            sample_count: video_samples,
            output_kind: kind,
            voice_gain_db: config.voice_gain_db,
            background_gain_db: config.background_gain_db,
            original: envelope
                .as_ref()
                .map(|_| Path::new(&config.video.asset_path)),
            envelope: envelope.as_deref(),
        },
        cancellation,
    )?;
    Ok(mixed)
}
