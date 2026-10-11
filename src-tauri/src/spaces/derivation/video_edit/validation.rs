use uuid::Uuid;

use super::VideoEditProject;

pub(in crate::spaces) fn validate_project(
    project: &VideoEditProject,
    source_revision: &str,
) -> Result<(), String> {
    if project.version != 1 || source_revision.is_empty() {
        return Err("Video edit project version or source is invalid".into());
    }
    if project.source_revision != source_revision
        || !(1..=16_384).contains(&project.source_width)
        || !(1..=16_384).contains(&project.source_height)
        || project.source_width > project.source_height * 10
        || project.source_height > project.source_width * 10
        || project.video_track.clips.is_empty()
        || project.video_track.clips.len() > 500
        || project.text_track.items.len() > 500
    {
        return Err("Video edit project does not match the source".into());
    }
    if project.video_track.clips.iter().any(|clip| {
        Uuid::parse_str(&clip.id).is_err()
            || clip.end_ms <= clip.start_ms
            || !(0.0..=2.0).contains(&clip.volume)
    }) {
        return Err("Video edit project contains an invalid clip".into());
    }
    if project.text_track.items.iter().any(|item| {
        Uuid::parse_str(&item.id).is_err()
            || item.end_ms <= item.start_ms
            || item.text.trim().is_empty()
            || item.text.len() > 500
            || !(16..=96).contains(&item.style.size)
            || item.style.color.len() != 7
            || !item.style.color.starts_with('#')
            || !item.style.color[1..]
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit())
    }) {
        return Err("Video edit project contains invalid timed text".into());
    }
    if !matches!(project.transform.rotation, 0 | 90 | 180 | 270) {
        return Err("Video rotation is invalid".into());
    }
    Ok(())
}
