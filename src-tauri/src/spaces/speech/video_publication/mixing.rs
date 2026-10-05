use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use tokio_util::sync::CancellationToken;

use super::super::{
    contracts::SpeechSegmentRenderOutput, publication::hash_file,
    video_config::VideoDubbingOutputConfig,
};
use super::{
    final_mix::{input_labels, mix_final, FinalMixSpec, MixOutput},
    wav_stream, MAX_MIX_INPUTS, SAMPLE_RATE,
};

pub(in crate::spaces::speech) struct TimedRender {
    render: SpeechSegmentRenderOutput,
    start_sample: u64,
    end_sample: u64,
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
            let start = u64::try_from(start)
                .map_err(|_| "Video target start is invalid")?
                .checked_mul(u64::from(SAMPLE_RATE))
                .ok_or("Video target start overflow")?
                / 1_000;
            let mut end = u64::try_from(end)
                .map_err(|_| "Video target end is invalid")?
                .checked_mul(u64::from(SAMPLE_RATE))
                .ok_or("Video target end overflow")?
                / 1_000;
            if end > video_samples && end - video_samples <= u64::from(SAMPLE_RATE) / 1_000 {
                end = video_samples;
            }
            if start >= end || end > video_samples {
                return Err("Video target window exceeds the verified source duration".into());
            }
            Ok(TimedRender {
                render,
                start_sample: start,
                end_sample: end,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    timed.sort_by_key(|item| (item.start_sample, item.end_sample));
    Ok(timed)
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
    let mut fitted = Vec::with_capacity(timed.len());
    for (index, item) in timed.iter().enumerate() {
        fitted.push(fit_render(ffmpeg, work, index, item, cancellation)?);
    }
    let lanes = assign_lanes(timed);
    let mut lane_paths = Vec::with_capacity(lanes.len());
    for (index, lane) in lanes.iter().enumerate() {
        if cancellation.is_cancelled() {
            return Err("cancelled".into());
        }
        let path = work.join(format!("lane-{index}.wav"));
        write_lane(&path, lane, timed, &fitted, video_samples, cancellation)?;
        if wav_stream::sample_count(&path, 1)? != video_samples {
            return Err("Video voice lane has an invalid duration".into());
        }
        lane_paths.push(path);
    }
    let voice_inputs = reduce_mix_inputs(ffmpeg, work, lane_paths, cancellation)?;
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
        &voice_inputs,
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

fn cleanup_failure(mut cause: String, path: &Path) -> String {
    if let Err(error) = fs::remove_file(path) {
        if error.kind() != std::io::ErrorKind::NotFound {
            cause.push_str(&format!("; failed to remove {}: {error}", path.display()));
        }
    }
    cause
}

fn fit_render(
    ffmpeg: &Path,
    work: &Path,
    index: usize,
    item: &TimedRender,
    cancellation: &CancellationToken,
) -> Result<PathBuf, String> {
    let source = Path::new(&item.render.path);
    if hash_file(source)? != item.render.sha256 {
        return Err("Video speech segment digest differs from its render metadata".into());
    }
    let samples = item.end_sample - item.start_sample;
    let path = work.join(format!("fit-{index}.wav"));
    let tempo = (item.render.sample_count as f64 * f64::from(SAMPLE_RATE))
        / (f64::from(item.render.sample_rate) * samples as f64);
    if !(0.9..=1.1).contains(&tempo) {
        return Err("Video speech segment no longer fits its target window".into());
    }
    let tempo_filter = if tempo == 1.0 {
        String::new()
    } else {
        format!("atempo={tempo:.9},")
    };
    let filter = format!(
        "aresample={SAMPLE_RATE},{tempo_filter}apad,atrim=end_sample={samples},aformat=sample_fmts=flt:channel_layouts=mono"
    );
    super::media_pipeline::run_ffmpeg(
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
    if wav_stream::sample_count(&path, 1)? != samples {
        return Err(cleanup_failure(
            "Fitted speech does not match its target sample count".into(),
            &path,
        ));
    }
    Ok(path)
}

fn assign_lanes(timed: &[TimedRender]) -> Vec<Vec<usize>> {
    let mut available = BinaryHeap::<Reverse<(u64, usize)>>::new();
    let mut lanes: Vec<Vec<usize>> = Vec::new();
    for (index, item) in timed.iter().enumerate() {
        let lane = if let Some(Reverse((end, lane))) = available.peek().copied() {
            if end <= item.start_sample {
                available.pop();
                lane
            } else {
                lanes.len()
            }
        } else {
            0
        };
        if lane == lanes.len() {
            lanes.push(Vec::new());
        }
        lanes[lane].push(index);
        available.push(Reverse((item.end_sample, lane)));
    }
    lanes
}

fn write_lane(
    path: &Path,
    lane: &[usize],
    timed: &[TimedRender],
    fitted: &[PathBuf],
    video_samples: u64,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let part = path.with_extension("wav.part");
    let data_bytes = video_samples
        .checked_mul(4)
        .ok_or("Video voice lane size overflow")?;
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&part)
        .map_err(|error| error.to_string())?;
    wav_stream::write_header(&mut output, data_bytes)?;
    let mut cursor = 0_u64;
    for index in lane {
        wav_stream::write_silence(
            &mut output,
            timed[*index].start_sample - cursor,
            cancellation,
        )?;
        wav_stream::copy_samples(&fitted[*index], &mut output, cancellation)?;
        cursor = timed[*index].end_sample;
    }
    wav_stream::write_silence(&mut output, video_samples - cursor, cancellation)?;
    output.sync_all().map_err(|error| error.to_string())?;
    fs::rename(&part, path).map_err(|error| error.to_string())
}

fn reduce_mix_inputs(
    ffmpeg: &Path,
    work: &Path,
    mut inputs: Vec<PathBuf>,
    cancellation: &CancellationToken,
) -> Result<Vec<PathBuf>, String> {
    let mut round = 0;
    while inputs.len() > MAX_MIX_INPUTS {
        let mut next = Vec::new();
        for (group, chunk) in inputs.chunks(MAX_MIX_INPUTS).enumerate() {
            let output = work.join(format!("voice-{round}-{group}.caf"));
            mix_group(ffmpeg, chunk, &output, cancellation)?;
            next.push(output);
        }
        inputs = next;
        round += 1;
    }
    Ok(inputs)
}

fn mix_group(
    ffmpeg: &Path,
    inputs: &[PathBuf],
    output: &Path,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let mut command = Command::new(ffmpeg);
    command.args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y"]);
    for input in inputs {
        command.arg("-i").arg(input);
    }
    command
        .arg("-filter_complex")
        .arg(format!(
            "{}amix=inputs={}:duration=longest:normalize=0[out]",
            input_labels(inputs.len()),
            inputs.len(),
        ))
        .args(["-map", "[out]", "-c:a", "pcm_f32le", "-f", "caf"])
        .arg(output);
    super::media_pipeline::run_ffmpeg(
        &mut command,
        cancellation,
        "FFmpeg could not combine video voice lanes",
    )
}
