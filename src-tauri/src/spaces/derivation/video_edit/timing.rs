use super::VideoEditProject;

pub(super) fn duration_ms(project: &VideoEditProject) -> u64 {
    project
        .video_track
        .clips
        .iter()
        .map(|clip| {
            clip.end_ms
                - clip.start_ms
                - clip
                    .transition
                    .as_ref()
                    .map_or(0, |transition| transition.duration_ms)
        })
        .sum()
}
pub(super) fn seconds(milliseconds: u64) -> String {
    format!("{:.3}", milliseconds as f64 / 1000.0)
}
