use super::{
    clip_filter::asset,
    contracts::VideoAssetKind,
    source_audio,
    start::VideoEditSource,
    timing::{duration_ms, seconds},
};
use std::{collections::HashMap, path::Path};
use tokio_util::sync::CancellationToken;

/// Input order is the contract the filter graph relies on: visual clips, audio clips, then the
/// generated project background.
pub(super) fn prepare(
    ffmpeg: &Path,
    source: &VideoEditSource,
    cancellation: &CancellationToken,
) -> Result<(Vec<String>, Vec<bool>), String> {
    let project = &source.project;
    let mut args = Vec::new();
    let mut audio = Vec::new();
    let mut inspected = HashMap::new();
    for clip in &project.video_track.clips {
        let media = asset(project, &clip.asset_id)?;
        let path = path(source, &clip.asset_id)?;
        if media.kind == VideoAssetKind::Image {
            args.extend(["-loop".into(), "1".into()]);
        } else {
            args.extend(["-ss".into(), seconds(clip.source_in_ms)]);
        }
        args.extend([
            "-t".into(),
            seconds(clip.length_ms()),
            "-i".into(),
            path.into(),
        ]);
        let has_audio = if media.kind == VideoAssetKind::Video {
            match inspected.get(path) {
                Some(value) => *value,
                None => {
                    let value = source_audio::inspect(ffmpeg, path, cancellation)?;
                    inspected.insert(path, value);
                    value
                }
            }
        } else {
            false
        };
        audio.push(has_audio);
    }
    for clip in &project.audio_track.clips {
        args.extend([
            "-ss".into(),
            seconds(clip.source_in_ms),
            "-t".into(),
            seconds(clip.length_ms()),
            "-i".into(),
            path(source, &clip.asset_id)?.into(),
        ]);
    }
    let background = project.output.background.trim_start_matches('#');
    args.extend([
        "-t".into(),
        seconds(duration_ms(project)),
        "-f".into(),
        "lavfi".into(),
        "-i".into(),
        format!(
            "color=c=0x{background}:s={width}x{height}:r={rate}",
            width = project.output.width,
            height = project.output.height,
            rate = project.output.frame_rate
        ),
    ]);
    Ok((args, audio))
}

fn path<'a>(source: &'a VideoEditSource, id: &str) -> Result<&'a str, String> {
    source
        .assets
        .iter()
        .find(|asset| asset.node_id == id)
        .map(|asset| asset.path.as_str())
        .ok_or_else(|| "Video material was not resolved".into())
}
