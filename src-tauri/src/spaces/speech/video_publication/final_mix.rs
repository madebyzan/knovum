use super::SAMPLE_RATE;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::spaces::speech) enum MixOutput {
    Combined,
    Dialogue,
    Background,
}

pub(super) struct FinalMixSpec<'a> {
    pub(super) sample_count: u64,
    pub(super) voice_gain_db: f32,
    pub(super) background_gain_db: f32,
    pub(super) original: Option<&'a Path>,
    pub(super) envelope: Option<&'a Path>,
    pub(super) output_kind: MixOutput,
}

pub(super) fn mix_final(
    ffmpeg: &Path,
    voices: &[PathBuf],
    background: Option<&Path>,
    output: &Path,
    spec: FinalMixSpec<'_>,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let FinalMixSpec {
        sample_count,
        voice_gain_db,
        background_gain_db,
        original,
        envelope,
        output_kind,
    } = spec;
    let mut command = Command::new(ffmpeg);
    command.args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y"]);
    for input in voices {
        command.arg("-i").arg(input);
    }
    if let Some(background) = background {
        command.arg("-i").arg(background);
    }
    if let (Some(original), Some(envelope)) = (original, envelope) {
        command.arg("-i").arg(original).arg("-i").arg(envelope);
    }
    let voice = if voices.is_empty() {
        String::new()
    } else {
        format!(
            "{}amix=inputs={}:duration=longest:normalize=0,volume={voice_gain_db}dB[voice];",
            input_labels(voices.len()),
            voices.len()
        )
    };
    let bed = if background.is_some() {
        let index = voices.len();
        let separated = format!("[{index}:a:0]aresample={SAMPLE_RATE},aformat=channel_layouts=stereo,volume={background_gain_db}dB,apad,atrim=end_sample={sample_count}");
        if original.is_some() {
            format!("{separated}[bed];[{}:a:0]aresample={SAMPLE_RATE},aformat=channel_layouts=stereo,apad,atrim=end_sample={sample_count}[original];[{}:a:0]pan=stereo|c0=c0|c1=c0,asplit=2[inside][invert];[invert]aeval=exprs='1-val(0)|1-val(1)'[outside];[bed][inside]amultiply[separated];[original][outside]amultiply[ambient];[separated][ambient]amix=inputs=2:normalize=0[background];", index + 1, index + 2)
        } else {
            format!("{separated}[background];")
        }
    } else {
        String::new()
    };
    let filter = if output_kind == MixOutput::Background {
        format!("{bed}[background]aformat=sample_fmts=flt:channel_layouts=stereo[out]")
    } else if output_kind == MixOutput::Dialogue {
        format!("{voice}[voice]atrim=end_sample={sample_count},aformat=sample_fmts=flt:channel_layouts=mono[out]")
    } else if background.is_some() {
        format!("{voice}{bed}[voice]pan=stereo|c0=c0|c1=c0[dialogue];[dialogue][background]amix=inputs=2:duration=longest:normalize=0,alimiter=limit=0.95:level=false:latency=true,atrim=end_sample={sample_count},aformat=sample_fmts=flt:channel_layouts=stereo[out]")
    } else {
        format!("{voice}[voice]alimiter=limit=0.95:level=false:latency=true,atrim=end_sample={sample_count},aformat=sample_fmts=flt:channel_layouts=mono[out]")
    };
    command
        .arg("-filter_complex")
        .arg(filter)
        .args(["-map", "[out]", "-c:a", "pcm_f32le", "-f", "wav"])
        .arg(output);
    crate::media::ffmpeg::run(
        &mut command,
        cancellation,
        "FFmpeg could not mix video dubbing audio",
    )?;
    if super::policy::wav_sample_count(output, if background.is_some() { 2 } else { 1 })?
        != sample_count
    {
        return Err("Mixed video dubbing audio has an invalid duration".into());
    }
    Ok(())
}

pub(super) fn input_labels(count: usize) -> String {
    (0..count).fold(String::new(), |mut labels, index| {
        labels.push_str(&format!("[{index}:a:0]"));
        labels
    })
}
