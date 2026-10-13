use super::contracts::{VideoClip, VideoTransition};
use super::VideoEditProject;

pub(super) fn duration_ms(project: &VideoEditProject) -> u64 {
    project
        .video_track
        .clips
        .iter()
        .map(VideoClip::timeline_end_ms)
        .max()
        .unwrap_or(0)
}

pub(super) fn seconds(milliseconds: u64) -> String {
    format!("{:.3}", milliseconds as f64 / 1000.0)
}

pub(super) fn transition_from<'a>(
    project: &'a VideoEditProject,
    clip_id: &str,
) -> Option<&'a VideoTransition> {
    project
        .transitions
        .iter()
        .find(|transition| transition.from_clip_id == clip_id)
}

/// Timeline assembly order: a background gap, or the clip at that index.
pub(super) enum Segment {
    Gap { duration_ms: u64 },
    Clip { index: usize },
}

/// Clips are validated as ordered and non-overlapping beyond their transitions, so the assembly
/// order is a single pass over the track.
pub(super) fn segments(project: &VideoEditProject) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut cursor_ms = 0;
    for (index, clip) in project.video_track.clips.iter().enumerate() {
        if clip.timeline_start_ms > cursor_ms {
            segments.push(Segment::Gap {
                duration_ms: clip.timeline_start_ms - cursor_ms,
            });
        }
        segments.push(Segment::Clip { index });
        cursor_ms = clip.timeline_end_ms();
    }
    segments
}
