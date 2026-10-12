use super::contracts::{VideoAsset, VideoClip, VideoEditProject, VideoFit};
use super::timing::seconds;

pub(super) fn video(project: &VideoEditProject, clip: &VideoClip, index: usize) -> String {
    let mut filters = vec![
        format!("trim=duration={}", seconds(clip.end_ms - clip.start_ms)),
        "setpts=PTS-STARTPTS".into(),
    ];
    match clip.transform.rotation {
        90 => filters.push("transpose=clock".into()),
        180 => filters.extend(["hflip".into(), "vflip".into()]),
        270 => filters.push("transpose=cclock".into()),
        _ => {}
    }
    if clip.transform.flip_horizontal {
        filters.push("hflip".into());
    }
    if clip.transform.flip_vertical {
        filters.push("vflip".into());
    }
    let width = ((f64::from(project.output.width) * clip.transform.zoom / 2.0).ceil() * 2.0) as u32;
    let height =
        ((f64::from(project.output.height) * clip.transform.zoom / 2.0).ceil() * 2.0) as u32;
    let fit = match clip.transform.fit {
        VideoFit::Contain => "decrease",
        VideoFit::Cover => "increase",
    };
    filters.push(format!(
        "scale=w={width}:h={height}:force_original_aspect_ratio={fit}:force_divisible_by=2"
    ));
    filters.push(format!(r"pad=w=max(iw\,{w}):h=max(ih\,{h}):x=(ow-iw)/2:y=(oh-ih)/2:color=black,crop={w}:{h},setsar=1,fps={fps},settb=AVTB,format=yuv420p", w=project.output.width, h=project.output.height, fps=project.output.frame_rate));
    if clip.fade_in_ms > 0 {
        filters.push(format!("fade=t=in:st=0:d={}", seconds(clip.fade_in_ms)));
    }
    if clip.fade_out_ms > 0 {
        filters.push(format!(
            "fade=t=out:st={}:d={}",
            seconds(clip.end_ms - clip.start_ms - clip.fade_out_ms),
            seconds(clip.fade_out_ms)
        ));
    }
    format!("[{index}:v]{}[v{index}]", filters.join(","))
}
pub(super) fn audio(clip: &VideoClip, index: usize, has_audio: bool) -> String {
    let duration = seconds(clip.end_ms - clip.start_ms);
    let source = if has_audio {
        format!("[{index}:a]atrim=duration={duration},asetpts=PTS-STARTPTS,aresample=48000,aformat=sample_fmts=fltp:channel_layouts=stereo,apad,atrim=duration={duration}")
    } else {
        format!("anullsrc=r=48000:cl=stereo,atrim=duration={duration}")
    };
    format!(
        "{source},{}[a{index}]",
        audio_effects(
            clip.volume,
            clip.fade_in_ms,
            clip.fade_out_ms,
            clip.end_ms - clip.start_ms
        )
    )
}
pub(super) fn audio_effects(volume: f64, fade_in: u64, fade_out: u64, duration: u64) -> String {
    let mut filters = vec![format!("volume={volume}")];
    if fade_in > 0 {
        filters.push(format!("afade=t=in:st=0:d={}", seconds(fade_in)));
    }
    if fade_out > 0 {
        filters.push(format!(
            "afade=t=out:st={}:d={}",
            seconds(duration - fade_out),
            seconds(fade_out)
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
