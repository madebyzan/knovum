use super::{
    clip_filter,
    contracts::VideoTransitionKind,
    timing::{duration_ms, seconds},
    VideoEditProject,
};

pub(super) fn build(
    project: &VideoEditProject,
    audio: &[bool],
    ass_file: Option<&str>,
) -> Result<String, String> {
    if audio.len() != project.video_track.clips.len() {
        return Err("Video audio inspection is incomplete".into());
    }
    let mut filters = Vec::new();
    for (index, clip) in project.video_track.clips.iter().enumerate() {
        filters.push(clip_filter::video(project, clip, index));
        filters.push(clip_filter::audio(clip, index, audio[index]));
    }
    let (mut video, mut sound, mut duration) = (
        "v0".to_owned(),
        "a0".to_owned(),
        project.video_track.clips[0].end_ms - project.video_track.clips[0].start_ms,
    );
    for (index, previous) in project
        .video_track
        .clips
        .iter()
        .enumerate()
        .skip(1)
        .map(|(i, _)| (i, &project.video_track.clips[i - 1]))
    {
        let next_video = format!("joinedv{index}");
        let next_audio = format!("joineda{index}");
        if let Some(transition) = &previous.transition {
            let kind = match transition.kind {
                VideoTransitionKind::Dissolve => "fade",
                VideoTransitionKind::WipeLeft => "wipeleft",
                VideoTransitionKind::WipeRight => "wiperight",
            };
            filters.push(format!(
                "[{video}][v{index}]xfade=transition={kind}:duration={}:offset={}[{next_video}]",
                seconds(transition.duration_ms),
                seconds(duration - transition.duration_ms)
            ));
            filters.push(format!(
                "[{sound}][a{index}]acrossfade=d={}:c1=tri:c2=tri[{next_audio}]",
                seconds(transition.duration_ms)
            ));
            duration -= transition.duration_ms;
        } else {
            filters.push(format!(
                "[{video}][v{index}]concat=n=2:v=1:a=0,fps={},settb=AVTB[{next_video}]",
                project.output.frame_rate
            ));
            filters.push(format!(
                "[{sound}][a{index}]concat=n=2:v=0:a=1[{next_audio}]"
            ));
        }
        duration +=
            project.video_track.clips[index].end_ms - project.video_track.clips[index].start_ms;
        video = next_video;
        sound = next_audio;
    }
    let visual = match ass_file {
        Some(file) => format!("ass=filename={file}"),
        None if project.text_track.items.is_empty() => "null".into(),
        None => return Err("Video edit subtitle track is missing".into()),
    };
    filters.push(format!("[{video}]{visual}[outv]"));
    let mut mix = format!("[{sound}]");
    for (index, clip) in project.audio_track.clips.iter().enumerate() {
        let input = project.video_track.clips.len() + index;
        let effects = clip_filter::audio_effects(
            clip.volume,
            clip.fade_in_ms,
            clip.fade_out_ms,
            clip.end_ms - clip.start_ms,
        );
        filters.push(format!("[{input}:a]atrim=duration={},asetpts=PTS-STARTPTS,aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo,{effects},adelay={}:all=1,apad,atrim=duration={}[music{index}]", seconds(clip.end_ms-clip.start_ms), clip.timeline_start_ms, seconds(duration_ms(project))));
        mix.push_str(&format!("[music{index}]"));
    }
    if project.audio_track.clips.is_empty() {
        filters.push(format!("[{sound}]anull[outa]"));
    } else {
        filters.push(format!(
            "{mix}amix=inputs={}:duration=first:normalize=0[outa]",
            project.audio_track.clips.len() + 1
        ));
    }
    Ok(filters.join(";"))
}
