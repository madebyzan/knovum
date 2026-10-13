use super::clip_filter::{self, AudioSegment};
use super::contracts::VideoTransition;
use super::timing::{seconds, segments, Segment};
use super::VideoEditProject;

/// Builds the complete filter graph: every clip becomes a full-frame segment over the project
/// background, segments are joined by cuts or transitions, and all audio is mixed at its absolute
/// timeline position.
pub(super) fn build(
    project: &VideoEditProject,
    audio_present: &[bool],
    ass_file: Option<&str>,
) -> Result<String, String> {
    if audio_present.len() != project.video_track.clips.len() {
        return Err("Video audio inspection is incomplete".into());
    }
    let order = segments(project);
    let mut filters = vec![background_split(
        project.video_track.clips.len() + project.audio_track.clips.len(),
        order.len(),
    )];
    let assembly = describe(project, &order, &mut filters)?;
    let visual = join(project, &assembly, &mut filters);
    filters.push(match ass_file {
        Some(file) => format!("[{visual}]ass=filename={file}[outv]"),
        None if project.text_track.items.is_empty() => format!("[{visual}]null[outv]"),
        None => return Err("Video edit subtitle track is missing".into()),
    });
    filters.push(mix(project, audio_present));
    Ok(filters.join(";"))
}

struct AssemblySegment<'a> {
    label: String,
    length_ms: u64,
    /// Transition handed to the following segment, present only on a clip with an outgoing overlap.
    transition: Option<&'a VideoTransition>,
}

fn describe<'a>(
    project: &'a VideoEditProject,
    order: &[Segment],
    filters: &mut Vec<String>,
) -> Result<Vec<AssemblySegment<'a>>, String> {
    let mut assembly: Vec<AssemblySegment<'a>> = Vec::new();
    for segment in order {
        let background = format!("bg{}", assembly.len());
        match segment {
            Segment::Clip { index } => {
                let clip = &project.video_track.clips[*index];
                filters.push(clip_filter::video(project, clip, *index, &background)?);
                assembly.push(AssemblySegment {
                    label: format!("v{index}"),
                    length_ms: clip.length_ms(),
                    transition: super::timing::transition_from(project, &clip.id),
                });
            }
            Segment::Gap { duration_ms } => {
                let label = format!("hole{}", assembly.len());
                filters.push(clip_filter::gap(project, *duration_ms, &background, &label));
                assembly.push(AssemblySegment {
                    label,
                    length_ms: *duration_ms,
                    transition: None,
                });
            }
        }
    }
    Ok(assembly)
}

fn background_split(input: usize, branches: usize) -> String {
    let labels = (0..branches)
        .map(|index| format!("[bg{index}]"))
        .collect::<String>();
    format!("[{input}:v]split={branches}{labels}")
}

/// Folds the ordered segments into one stream. A transition consumes its overlap, so the running
/// offset is the assembled duration rather than the sum of clip lengths.
fn join(
    project: &VideoEditProject,
    assembly: &[AssemblySegment<'_>],
    filters: &mut Vec<String>,
) -> String {
    let Some(first) = assembly.first() else {
        return String::new();
    };
    let mut current = first.label.clone();
    let mut accumulated = first.length_ms;
    let mut carried = first.transition;
    for (position, segment) in assembly.iter().enumerate().skip(1) {
        let joined = format!("join{position}");
        match carried {
            Some(transition) => {
                filters.push(format!(
                    "[{current}][{next}]xfade=transition={kind}:duration={length}:offset={offset}[{joined}]",
                    next = segment.label,
                    kind = transition.kind.xfade_name(),
                    length = seconds(transition.duration_ms),
                    offset = seconds(accumulated - transition.duration_ms)
                ));
                accumulated += segment.length_ms - transition.duration_ms;
            }
            None => {
                filters.push(format!(
                    "[{current}][{next}]concat=n=2:v=1:a=0,{clock}[{joined}]",
                    next = segment.label,
                    clock = clip_filter::frame_clock(project)
                ));
                accumulated += segment.length_ms;
            }
        }
        current = joined;
        carried = segment.transition;
    }
    current
}

/// Mixes clip source audio and independent audio clips at their absolute positions. Transition
/// overlaps add an automatic audio crossfade so the mix matches the visual join.
fn mix(project: &VideoEditProject, audio_present: &[bool]) -> String {
    let mut filters = Vec::new();
    let mut labels = Vec::new();
    for (index, clip) in project.video_track.clips.iter().enumerate() {
        if !audio_present[index] {
            continue;
        }
        let incoming = index
            .checked_sub(1)
            .and_then(|previous| {
                super::timing::transition_from(project, &project.video_track.clips[previous].id)
            })
            .map_or(0, |transition| transition.duration_ms);
        let outgoing = super::timing::transition_from(project, &clip.id)
            .map_or(0, |transition| transition.duration_ms);
        let label = format!("a{index}");
        filters.push(clip_filter::audio(&AudioSegment {
            input: index,
            label: &label,
            length_ms: clip.length_ms(),
            timeline_start_ms: clip.timeline_start_ms,
            volume: clip.volume,
            fades: (
                clip.fade_in_ms.max(incoming),
                clip.fade_out_ms.max(outgoing),
            ),
        }));
        labels.push(label);
    }
    for (position, clip) in project.audio_track.clips.iter().enumerate() {
        let label = format!("music{position}");
        filters.push(clip_filter::audio(&AudioSegment {
            input: project.video_track.clips.len() + position,
            label: &label,
            length_ms: clip.length_ms(),
            timeline_start_ms: clip.timeline_start_ms,
            volume: clip.volume,
            fades: (clip.fade_in_ms, clip.fade_out_ms),
        }));
        labels.push(label);
    }
    filters.push(combine(
        &labels,
        seconds(super::timing::duration_ms(project)),
    ));
    filters.join(";")
}

fn combine(labels: &[String], duration: String) -> String {
    match labels.len() {
        0 => {
            format!("anullsrc=r=48000:cl=stereo,atrim=duration={duration},asetpts=PTS-STARTPTS[outa]")
        }
        1 => format!(
            "[{source}]apad,atrim=duration={duration}[outa]",
            source = labels[0]
        ),
        count => format!(
            "{inputs}amix=inputs={count}:duration=longest:normalize=0,apad,atrim=duration={duration}[outa]",
            inputs = labels
                .iter()
                .map(|label| format!("[{label}]"))
                .collect::<String>()
        ),
    }
}
