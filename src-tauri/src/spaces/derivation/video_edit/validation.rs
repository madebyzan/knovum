use std::collections::HashSet;
use uuid::Uuid;

use super::contracts::{VideoAsset, VideoAssetKind, VideoClip, VideoEditProject};

pub(in crate::spaces) fn validate_project(
    project: &VideoEditProject,
    revision: &str,
) -> Result<(), String> {
    if project.version != 1 || revision.is_empty() || project.source_revision != revision {
        return Err("Video edit project version or source is invalid".into());
    }
    let output = &project.output;
    if !(2..=16_384).contains(&output.width)
        || !(2..=16_384).contains(&output.height)
        || output.width % 2 != 0
        || output.height % 2 != 0
        || !matches!(output.frame_rate, 24 | 25 | 30 | 60)
        || u64::from(output.width) * u64::from(output.height) > 33_177_600
    {
        return Err("Video output dimensions or frame rate are invalid".into());
    }
    if project.assets.is_empty()
        || project.assets.len() > 100
        || project.video_track.clips.is_empty()
        || project.video_track.clips.len() > 500
        || project.audio_track.clips.len() > 100
        || project.text_track.items.len() > 500
    {
        return Err("Video project exceeds its media limits".into());
    }
    let mut asset_ids = HashSet::new();
    for asset in &project.assets {
        validate_asset(asset)?;
        if !asset_ids.insert(&asset.node_id) {
            return Err("Video asset identities must be unique".into());
        }
    }
    if project.assets[0].kind != VideoAssetKind::Video
        || project.assets[0].source_revision != revision
    {
        return Err("Video project must retain its owning source".into());
    }
    let mut ids = HashSet::new();
    for (index, clip) in project.video_track.clips.iter().enumerate() {
        validate_clip(project, clip, index)?;
        if !ids.insert(&clip.id) {
            return Err("Video clip identities must be unique".into());
        }
    }
    let duration = super::timing::duration_ms(project);
    for clip in &project.audio_track.clips {
        let asset = project
            .assets
            .iter()
            .find(|asset| asset.node_id == clip.asset_id)
            .ok_or("Audio asset is missing")?;
        validate_interval(
            &clip.id,
            clip.start_ms,
            clip.end_ms,
            clip.volume,
            clip.fade_in_ms,
            clip.fade_out_ms,
        )?;
        if asset.kind != VideoAssetKind::Audio
            || clip.end_ms > asset.duration_ms
            || clip.timeline_start_ms > duration
            || clip.end_ms - clip.start_ms > duration - clip.timeline_start_ms
            || !ids.insert(&clip.id)
        {
            return Err("Audio clip source or timing is invalid".into());
        }
    }
    for item in &project.text_track.items {
        if Uuid::parse_str(&item.id).is_err()
            || !ids.insert(&item.id)
            || item.end_ms <= item.start_ms
            || item.end_ms > duration
            || item.text.trim().is_empty()
            || item.text.encode_utf16().count() > 500
            || !(16..=96).contains(&item.style.size)
            || !(100..=900).contains(&item.style.weight)
            || item.style.family.is_empty()
            || item.style.family.encode_utf16().count() > 128
            || item.style.color.len() != 7
            || !item.style.color.starts_with('#')
            || !item.style.color[1..]
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
        {
            return Err("Video edit project contains invalid timed text".into());
        }
    }
    Ok(())
}
fn validate_asset(asset: &VideoAsset) -> Result<(), String> {
    if asset.node_id.is_empty()
        || asset.node_id.len() > 128
        || asset.source_revision.is_empty()
        || asset.source_revision.len() > 128
        || asset.duration_ms > 86_400_000
        || asset.width > 16_384
        || asset.height > 16_384
    {
        return Err("Video asset identity or metadata is invalid".into());
    }
    let dimensions = match asset.kind {
        VideoAssetKind::Audio => asset.width == 0 && asset.height == 0 && asset.duration_ms > 0,
        VideoAssetKind::Video => asset.width > 0 && asset.height > 0 && asset.duration_ms > 0,
        VideoAssetKind::Image => asset.width > 0 && asset.height > 0,
    };
    if !dimensions {
        return Err("Video asset dimensions or duration are invalid".into());
    }
    Ok(())
}
fn validate_clip(project: &VideoEditProject, clip: &VideoClip, index: usize) -> Result<(), String> {
    let asset = project
        .assets
        .iter()
        .find(|asset| asset.node_id == clip.asset_id)
        .ok_or("Video asset is missing")?;
    validate_interval(
        &clip.id,
        clip.start_ms,
        clip.end_ms,
        clip.volume,
        clip.fade_in_ms,
        clip.fade_out_ms,
    )?;
    if asset.kind == VideoAssetKind::Audio
        || (asset.kind == VideoAssetKind::Video && clip.end_ms > asset.duration_ms)
        || !matches!(clip.transform.rotation, 0 | 90 | 180 | 270)
        || !(1.0..=4.0).contains(&clip.transform.zoom)
    {
        return Err("Video clip source or transform is invalid".into());
    }
    let incoming = index
        .checked_sub(1)
        .and_then(|i| project.video_track.clips[i].transition.as_ref())
        .map_or(0, |t| t.duration_ms);
    let outgoing = clip.transition.as_ref().map_or(0, |t| t.duration_ms);
    if clip.transition.is_some()
        && (!(100..=5_000).contains(&outgoing) || index + 1 == project.video_track.clips.len())
    {
        return Err("Video transition boundary is invalid".into());
    }
    if incoming + outgoing > clip.end_ms - clip.start_ms - 100 {
        return Err("Transition overlaps exceed available clip media".into());
    }
    Ok(())
}
fn validate_interval(
    id: &str,
    start: u64,
    end: u64,
    volume: f64,
    fade_in: u64,
    fade_out: u64,
) -> Result<(), String> {
    if Uuid::parse_str(id).is_err()
        || end < start.saturating_add(100)
        || end > 86_400_000
        || !(0.0..=2.0).contains(&volume)
        || fade_in > end - start
        || fade_out > end - start - fade_in
    {
        return Err("Video clip interval, volume or fade is invalid".into());
    }
    Ok(())
}
