use super::VideoEditProject;

pub fn build(
    project: &VideoEditProject,
    has_audio: bool,
    ass_file: Option<&str>,
) -> Result<String, String> {
    let mut filters = Vec::new();
    let mut inputs = String::new();
    for (index, clip) in project.video_track.clips.iter().enumerate() {
        let start = seconds(clip.start_ms);
        let end = seconds(clip.end_ms);
        filters.push(format!(
            "[0:v]trim=start={start}:end={end},setpts=PTS-STARTPTS[v{index}]"
        ));
        inputs.push_str(&format!("[v{index}]"));
        if has_audio {
            filters.push(format!(
                "[0:a]atrim=start={start}:end={end},asetpts=PTS-STARTPTS,volume={}[a{index}]",
                clip.volume
            ));
            inputs.push_str(&format!("[a{index}]"));
        }
    }
    let count = project.video_track.clips.len();
    if has_audio {
        filters.push(format!("{inputs}concat=n={count}:v=1:a=1[joinedv][outa]"));
    } else {
        filters.push(format!("{inputs}concat=n={count}:v=1:a=0[joinedv]"));
    }
    filters.push(format!(
        "[joinedv]{}[outv]",
        visual_filters(project, ass_file)?
    ));
    Ok(filters.join(";"))
}

fn visual_filters(project: &VideoEditProject, ass_file: Option<&str>) -> Result<String, String> {
    let mut filters = rotation_filters(project.transform.rotation)?;
    if project.transform.flip_horizontal {
        filters.push("hflip".into());
    }
    if project.transform.flip_vertical {
        filters.push("vflip".into());
    }
    if let Some(file) = ass_file {
        filters.push(format!("ass=filename={file}"));
    } else if !project.text_track.items.is_empty() {
        return Err("Video edit subtitle track is missing".into());
    }
    if filters.is_empty() {
        filters.push("null".into());
    }
    Ok(filters.join(","))
}

fn rotation_filters(rotation: u16) -> Result<Vec<String>, String> {
    match rotation {
        0 => Ok(Vec::new()),
        90 => Ok(vec!["transpose=clock".into()]),
        180 => Ok(vec!["hflip".into(), "vflip".into()]),
        270 => Ok(vec!["transpose=cclock".into()]),
        _ => Err("Video rotation is invalid".into()),
    }
}

fn seconds(milliseconds: u64) -> String {
    format!("{:.3}", milliseconds as f64 / 1000.0)
}
