use std::collections::HashSet;
use uuid::Uuid;

use super::contracts::{
    TextOverlay, VideoAsset, VideoAssetKind, VideoClip, VideoEditProject, VideoTransform,
    PROJECT_VERSION,
};
use super::timing;

const MIN_CLIP_MS: u64 = 100;
const MAX_MEDIA_MS: u64 = 86_400_000;
const AUDIO_LANES: u8 = 4;

pub(in crate::spaces) fn validate_project(
    project: &VideoEditProject,
    revision: &str,
) -> Result<(), String> {
    if project.version != PROJECT_VERSION
        || revision.is_empty()
        || project.source_revision != revision
    {
        return Err("Video edit project version or source is invalid".into());
    }
    validate_output(project)?;
    validate_limits(project)?;
    let mut ids = HashSet::new();
    validate_assets(project, revision)?;
    let overlaps = transition_overlaps(project, &mut ids)?;
    validate_video_track(project, &overlaps, &mut ids)?;
    let duration = timing::duration_ms(project);
    validate_audio_track(project, duration, &mut ids)?;
    for item in &project.text_track.items {
        validate_text(item, duration)?;
        if !ids.insert(&item.id) {
            return Err("Video project identities must be unique".into());
        }
    }
    Ok(())
}

fn validate_output(project: &VideoEditProject) -> Result<(), String> {
    let output = &project.output;
    if !(2..=16_384).contains(&output.width)
        || !(2..=16_384).contains(&output.height)
        || output.width % 2 != 0
        || output.height % 2 != 0
        || !matches!(output.frame_rate, 24 | 25 | 30 | 60)
        || u64::from(output.width) * u64::from(output.height) > 33_177_600
        || !is_hex_color(&output.background)
    {
        return Err("Video output dimensions, frame rate or background are invalid".into());
    }
    Ok(())
}

fn validate_limits(project: &VideoEditProject) -> Result<(), String> {
    if project.assets.is_empty()
        || project.assets.len() > 100
        || project.video_track.clips.is_empty()
        || project.video_track.clips.len() > 500
        || project.transitions.len() >= project.video_track.clips.len()
        || project.audio_track.clips.len() > 100
        || project.text_track.items.len() > 500
    {
        return Err("Video project exceeds its media limits".into());
    }
    Ok(())
}

fn validate_assets(project: &VideoEditProject, revision: &str) -> Result<(), String> {
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
    Ok(())
}

/// Outgoing transition duration per clip index, with zero where a cut has no transition.
fn transition_overlaps<'a>(
    project: &'a VideoEditProject,
    ids: &mut HashSet<&'a String>,
) -> Result<Vec<u64>, String> {
    let clips = &project.video_track.clips;
    let mut overlaps = vec![0_u64; clips.len()];
    for transition in &project.transitions {
        if Uuid::parse_str(&transition.id).is_err() || !ids.insert(&transition.id) {
            return Err("Video project identities must be unique".into());
        }
        if !(MIN_CLIP_MS..=5_000).contains(&transition.duration_ms) {
            return Err("A transition must last between 100 ms and 5 s".into());
        }
        let index = clips
            .iter()
            .position(|clip| clip.id == transition.from_clip_id)
            .ok_or("A transition references a missing clip")?;
        if index + 1 == clips.len() {
            return Err("A transition needs a following clip".into());
        }
        if overlaps[index] != 0 {
            return Err("A clip boundary carries at most one transition".into());
        }
        overlaps[index] = transition.duration_ms;
    }
    for (index, clip) in clips.iter().enumerate() {
        let incoming = index
            .checked_sub(1)
            .map_or(0, |previous| overlaps[previous]);
        let outgoing = overlaps[index];
        if incoming + outgoing + MIN_CLIP_MS > clip.length_ms() {
            return Err("Transition overlaps exceed available clip media".into());
        }
    }
    Ok(overlaps)
}

fn validate_video_track<'a>(
    project: &'a VideoEditProject,
    overlaps: &[u64],
    ids: &mut HashSet<&'a String>,
) -> Result<(), String> {
    for (index, clip) in project.video_track.clips.iter().enumerate() {
        let asset = asset_of(project, &clip.asset_id)?;
        if asset.kind == VideoAssetKind::Audio {
            return Err("A visual clip cannot use an audio asset".into());
        }
        validate_source_window(
            clip.id.as_str(),
            (clip.source_in_ms, clip.source_out_ms),
            (clip.fade_in_ms, clip.fade_out_ms),
            clip.volume,
        )?;
        if asset.kind == VideoAssetKind::Video && clip.source_out_ms > asset.duration_ms {
            return Err("A visual clip exceeds its source media".into());
        }
        validate_transform(&clip.transform)?;
        if clip.timeline_end_ms() > MAX_MEDIA_MS {
            return Err("Video project exceeds the supported timeline length".into());
        }
        if let Some(previous) = index
            .checked_sub(1)
            .map(|at| &project.video_track.clips[at])
        {
            validate_placement(previous, clip, overlaps[index - 1])?;
        }
        if !ids.insert(&clip.id) {
            return Err("Video project identities must be unique".into());
        }
    }
    Ok(())
}

/// Clips are ordered, and two clips may only overlap by exactly the transition between them.
fn validate_placement(previous: &VideoClip, clip: &VideoClip, overlap: u64) -> Result<(), String> {
    if clip.timeline_start_ms <= previous.timeline_start_ms
        || previous
            .timeline_end_ms()
            .saturating_sub(clip.timeline_start_ms)
            != overlap
        || (overlap == 0 && clip.timeline_start_ms < previous.timeline_end_ms())
    {
        return Err("Clip overlap must match the transition between the two clips".into());
    }
    Ok(())
}

fn validate_audio_track<'a>(
    project: &'a VideoEditProject,
    duration: u64,
    ids: &mut HashSet<&'a String>,
) -> Result<(), String> {
    let mut lane_cursor = [0_u64; AUDIO_LANES as usize];
    for clip in &project.audio_track.clips {
        let asset = asset_of(project, &clip.asset_id)?;
        validate_source_window(
            clip.id.as_str(),
            (clip.source_in_ms, clip.source_out_ms),
            (clip.fade_in_ms, clip.fade_out_ms),
            clip.volume,
        )?;
        if asset.kind != VideoAssetKind::Audio
            || clip.source_out_ms > asset.duration_ms
            || clip.lane >= AUDIO_LANES
            || clip.timeline_start_ms + clip.length_ms() > duration
        {
            return Err("Audio clip source, lane or timing is invalid".into());
        }
        let cursor = &mut lane_cursor[usize::from(clip.lane)];
        if clip.timeline_start_ms < *cursor {
            return Err("Audio clips in one lane cannot overlap".into());
        }
        *cursor = clip.timeline_start_ms + clip.length_ms();
        if !ids.insert(&clip.id) {
            return Err("Video project identities must be unique".into());
        }
    }
    Ok(())
}

fn validate_text(item: &TextOverlay, duration: u64) -> Result<(), String> {
    let style = &item.style;
    if Uuid::parse_str(&item.id).is_err()
        || item.end_ms < item.start_ms.saturating_add(MIN_CLIP_MS)
        || item.end_ms > duration
        || item.text.trim().is_empty()
        || item.text.encode_utf16().count() > 500
        || !(-0.5..=1.5).contains(&item.x)
        || !(-0.5..=1.5).contains(&item.y)
        || !(12..=200).contains(&style.size)
        || !(100..=900).contains(&style.weight)
        || style.decoration_strength > 20
        || style.family.is_empty()
        || style.family.encode_utf16().count() > 128
        || !is_hex_color(&style.color)
        || !is_hex_color(&style.decoration_color)
    {
        return Err("Video edit project contains invalid timed text".into());
    }
    Ok(())
}

fn validate_asset(asset: &VideoAsset) -> Result<(), String> {
    if asset.node_id.is_empty()
        || asset.node_id.len() > 128
        || asset.source_revision.is_empty()
        || asset.source_revision.len() > 128
        || asset.duration_ms > MAX_MEDIA_MS
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

fn validate_transform(transform: &VideoTransform) -> Result<(), String> {
    if !(-2.0..=2.0).contains(&transform.x)
        || !(-2.0..=2.0).contains(&transform.y)
        || !(0.05..=8.0).contains(&transform.scale)
        || !(-180.0..=180.0).contains(&transform.rotation)
        || !(0.0..=1.0).contains(&transform.opacity)
    {
        return Err("Video clip transform is out of range".into());
    }
    Ok(())
}

/// Shared source-window invariants for visual and audio clips.
fn validate_source_window(
    id: &str,
    source: (u64, u64),
    fades: (u64, u64),
    volume: f64,
) -> Result<(), String> {
    let (start, end) = source;
    if Uuid::parse_str(id).is_err()
        || end < start.saturating_add(MIN_CLIP_MS)
        || end > MAX_MEDIA_MS
        || !(0.0..=2.0).contains(&volume)
        || fades.0 + fades.1 > end - start
    {
        return Err("Video clip interval, volume or fade is invalid".into());
    }
    Ok(())
}

fn asset_of<'a>(project: &'a VideoEditProject, id: &str) -> Result<&'a VideoAsset, String> {
    project
        .assets
        .iter()
        .find(|asset| asset.node_id == id)
        .ok_or_else(|| "Video clip asset is unavailable".into())
}

fn is_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
}
