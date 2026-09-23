use super::{TextPlacement, VideoEditProject};

pub fn build(
    project: &VideoEditProject,
    has_audio: bool,
    font_name: &str,
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
        visual_filters(project, font_name)?
    ));
    Ok(filters.join(";"))
}

fn visual_filters(project: &VideoEditProject, font_name: &str) -> Result<String, String> {
    let mut filters = rotation_filters(project.transform.rotation)?;
    if project.transform.flip_horizontal {
        filters.push("hflip".into());
    }
    if project.transform.flip_vertical {
        filters.push("vflip".into());
    }
    for item in &project.text_track.items {
        let y = match item.placement {
            TextPlacement::Top => "h*0.08",
            TextPlacement::Center => "(h-text_h)/2",
            TextPlacement::Bottom => "h-text_h-h*0.08",
        };
        filters.push(format!("drawtext=fontfile='{font_name}':text='{}':expansion=none:fontcolor=white:fontsize=h/24:borderw=2:bordercolor=black@0.75:x=(w-text_w)/2:y={y}:enable='between(t,{},{})'", escape_text(&item.text), seconds(item.start_ms), seconds(item.end_ms)));
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

fn escape_text(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace(':', "\\:")
}

fn seconds(milliseconds: u64) -> String {
    format!("{:.3}", milliseconds as f64 / 1000.0)
}
