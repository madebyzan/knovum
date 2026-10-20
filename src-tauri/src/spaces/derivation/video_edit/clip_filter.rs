use super::contracts::{VideoAsset, VideoClip, VideoEditOutput, VideoFit};
use super::timing::seconds;
use super::VideoEditProject;

/// Layer geometry in output pixels. The Renderer preview computes the same values, so the stage and
/// the exported frame place a clip identically.
pub(super) struct LayerGeometry {
    pub width: u32,
    pub height: u32,
    pub offset_x: i64,
    pub offset_y: i64,
}

pub(super) fn geometry(
    asset: &VideoAsset,
    clip: &VideoClip,
    output: &VideoEditOutput,
) -> LayerGeometry {
    let (source_width, source_height) = (f64::from(asset.width), f64::from(asset.height));
    let (frame_width, frame_height) = (f64::from(output.width), f64::from(output.height));
    let horizontal = frame_width / source_width;
    let vertical = frame_height / source_height;
    let base = match clip.transform.fit {
        VideoFit::Contain => horizontal.min(vertical),
        VideoFit::Cover => horizontal.max(vertical),
    };
    let ratio = base * clip.transform.scale;
    LayerGeometry {
        width: even(source_width * ratio),
        height: even(source_height * ratio),
        offset_x: (clip.transform.x * frame_width).round() as i64,
        offset_y: (clip.transform.y * frame_height).round() as i64,
    }
}

fn even(value: f64) -> u32 {
    let rounded = value.round().clamp(2.0, 16_384.0) as u32;
    rounded + rounded % 2
}

/// One visual clip composed over the project background as a full-frame segment.
pub(super) fn video(
    project: &VideoEditProject,
    clip: &VideoClip,
    index: usize,
    background: &str,
) -> Result<String, String> {
    let asset = asset(project, &clip.asset_id)?;
    let layer = geometry(asset, clip, &project.output);
    let length = clip.length_ms();
    let mut source = vec![
        format!("trim=duration={}", seconds(length)),
        "setpts=PTS-STARTPTS".to_owned(),
    ];
    if clip.transform.flip_horizontal {
        source.push("hflip".to_owned());
    }
    if clip.transform.flip_vertical {
        source.push("vflip".to_owned());
    }
    source.push(format!("scale=w={}:h={}", layer.width, layer.height));
    source.push("format=rgba".to_owned());
    if clip.transform.rotation.abs() > f64::EPSILON {
        let angle = clip.transform.rotation.to_radians();
        source.push(format!(
            "rotate=a={angle:.6}:ow=rotw({angle:.6}):oh=roth({angle:.6}):c=none"
        ));
    }
    if clip.transform.opacity < 1.0 {
        source.push(format!(
            "colorchannelmixer=aa={:.4}",
            clip.transform.opacity
        ));
    }
    let mut composed = vec![format!(
        "overlay=x=(main_w/2+{})-overlay_w/2:y=(main_h/2+{})-overlay_h/2:shortest=1:format=auto",
        layer.offset_x, layer.offset_y
    )];
    composed.push("setsar=1".to_owned());
    composed.push(frame_clock(project));
    composed.push("format=yuv420p".to_owned());
    composed.extend(visual_fades(clip, length));
    Ok(format!(
        "[{index}:v]{source}[vl{index}];[{background}][vl{index}]{composed}[v{index}]",
        source = source.join(","),
        composed = composed.join(",")
    ))
}

fn visual_fades(clip: &VideoClip, length: u64) -> Vec<String> {
    let mut fades = Vec::new();
    if clip.fade_in_ms > 0 {
        fades.push(format!("fade=t=in:st=0:d={}", seconds(clip.fade_in_ms)));
    }
    if clip.fade_out_ms > 0 {
        fades.push(format!(
            "fade=t=out:st={}:d={}",
            seconds(length - clip.fade_out_ms),
            seconds(clip.fade_out_ms)
        ));
    }
    fades
}

/// A timeline hole shows the project background for its whole length.
pub(super) fn gap(
    project: &VideoEditProject,
    duration_ms: u64,
    background: &str,
    label: &str,
) -> String {
    format!(
        "[{background}]trim=duration={duration},setpts=PTS-STARTPTS,setsar=1,{clock},format=yuv420p[{label}]",
        duration = seconds(duration_ms),
        clock = frame_clock(project)
    )
}

pub(super) fn frame_clock(project: &VideoEditProject) -> String {
    format!("fps={},settb=AVTB", project.output.frame_rate)
}

/// Timeline-placed audio for a visual clip's own track or an independent audio clip.
pub(super) struct AudioSegment<'a> {
    pub input: usize,
    pub label: &'a str,
    pub length_ms: u64,
    pub timeline_start_ms: u64,
    pub volume: f64,
    pub fades_ms: (u64, u64),
}

pub(super) fn audio(segment: &AudioSegment<'_>) -> String {
    let duration_seconds = seconds(segment.length_ms);
    let delay = if segment.timeline_start_ms == 0 {
        "anull".to_owned()
    } else {
        format!("adelay={}:all=1", segment.timeline_start_ms)
    };
    format!(
        "[{input}:a]atrim=duration={duration_seconds},asetpts=PTS-STARTPTS,aresample=48000,\
aformat=sample_fmts=fltp:channel_layouts=stereo,apad,atrim=duration={duration_seconds},{effects},{delay}[{label}]",
        input = segment.input,
        effects = audio_effects(
            segment.volume,
            segment.fades_ms.0,
            segment.fades_ms.1,
            segment.length_ms
        ),
        label = segment.label
    )
}

pub(super) fn audio_effects(
    volume: f64,
    fade_in_ms: u64,
    fade_out_ms: u64,
    duration_ms: u64,
) -> String {
    let mut filters = vec![format!("volume={volume}")];
    if fade_in_ms > 0 {
        filters.push(format!("afade=t=in:st=0:d={}", seconds(fade_in_ms)));
    }
    if fade_out_ms > 0 {
        filters.push(format!(
            "afade=t=out:st={}:d={}",
            seconds(duration_ms - fade_out_ms),
            seconds(fade_out_ms)
        ));
    }
    filters.join(",")
}

pub(super) fn asset<'a>(project: &'a VideoEditProject, id: &str) -> Result<&'a VideoAsset, String> {
    project
        .assets
        .iter()
        .find(|asset| asset.node_id == id)
        .ok_or_else(|| "Video clip asset is unavailable".into())
}
