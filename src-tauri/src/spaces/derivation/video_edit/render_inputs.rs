use super::{
    clip_filter::asset, contracts::VideoAssetKind, source_audio, start::VideoEditSource,
    timing::seconds,
};
use std::{collections::HashMap, path::Path};
use tokio_util::sync::CancellationToken;

pub(super) fn prepare(
    ffmpeg: &Path,
    source: &VideoEditSource,
    cancellation: &CancellationToken,
) -> Result<(Vec<String>, Vec<bool>), String> {
    let mut args = Vec::new();
    let mut audio = Vec::new();
    let mut inspected = HashMap::new();
    for clip in &source.project.video_track.clips {
        let media = asset(&source.project, &clip.asset_id)?;
        let path = path(source, &clip.asset_id)?;
        if media.kind == VideoAssetKind::Image {
            args.extend(["-loop".into(), "1".into()]);
        } else {
            args.extend(["-ss".into(), seconds(clip.start_ms)]);
        }
        args.extend([
            "-t".into(),
            seconds(clip.end_ms - clip.start_ms),
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
    for clip in &source.project.audio_track.clips {
        args.extend([
            "-ss".into(),
            seconds(clip.start_ms),
            "-t".into(),
            seconds(clip.end_ms - clip.start_ms),
            "-i".into(),
            path(source, &clip.asset_id)?.into(),
        ]);
    }
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
