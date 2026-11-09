use super::SAMPLE_RATE;
use std::{path::Path, process::Command};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::spaces::speech) enum MixOutput {
    Combined,
    Dialogue,
    Background,
}

/// Bus levels and blend sources for one mix artifact. Every gain is a static decibel value the
/// caller has already resolved from loudness measurements and user trims, so the mix graph itself
/// is deterministic and sample-exact. The program loudness ceiling is applied later by
/// [`super::mastering::master`]; this stage only sums buses.
pub(super) struct FinalMixSpec<'a> {
    pub(super) sample_count: u64,
    pub(super) dialogue_gain_db: f32,
    pub(super) background_gain_db: f32,
    pub(super) original_gain_db: f32,
    /// Envelope multiplied into the whole background bus under synthesized dialogue.
    pub(super) duck: Option<&'a Path>,
    /// The source video, whose audio is preserved outside dialogue windows.
    pub(super) original: Option<&'a Path>,
    /// The swap envelope selecting between bed and preserved original.
    pub(super) envelope: Option<&'a Path>,
    pub(super) output_kind: MixOutput,
}

pub(super) fn mix_final(
    ffmpeg: &Path,
    dialogue: Option<&Path>,
    background: Option<&Path>,
    output: &Path,
    spec: FinalMixSpec<'_>,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    let FinalMixSpec {
        sample_count,
        dialogue_gain_db,
        background_gain_db,
        original_gain_db,
        duck,
        original,
        envelope,
        output_kind,
    } = spec;
    let mut input_count = 0;
    let mut next_input = |present: bool| {
        let index = input_count;
        input_count += usize::from(present);
        index
    };
    let dialogue_input = dialogue.map(|path| (next_input(true), path));
    let background_input = background.map(|path| (next_input(true), path));
    let swap_input = original
        .zip(envelope)
        .map(|(original, envelope)| (next_input(true), next_input(true), original, envelope));
    let duck_input = duck.map(|path| (next_input(true), path));

    let mut command = Command::new(ffmpeg);
    command.args(["-hide_banner", "-loglevel", "error", "-nostdin", "-y"]);
    if let Some((_, path)) = dialogue_input {
        command.arg("-i").arg(path);
    }
    if let Some((_, path)) = background_input {
        command.arg("-i").arg(path);
    }
    if let Some((_, _, original, envelope)) = swap_input {
        command.arg("-i").arg(original).arg("-i").arg(envelope);
    }
    if let Some((_, path)) = duck_input {
        command.arg("-i").arg(path);
    }

    let mut graph = String::new();
    let voice = dialogue_input.map(|(index, _)| {
        graph.push_str(&format!(
            "[{index}:a:0]volume={dialogue_gain_db:.6}dB[voice];"
        ));
        "voice"
    });
    let bed = background_input.map(|(index, _)| {
        graph.push_str(&format!(
            "[{index}:a:0]aresample={SAMPLE_RATE},aformat=channel_layouts=stereo,volume={background_gain_db:.6}dB,apad,atrim=end_sample={sample_count}[bed];"
        ));
        let mut label = "bed";
        if let Some((original_index, envelope_index, _, _)) = swap_input {
            graph.push_str(&format!(
                "[{original_index}:a:0]aresample={SAMPLE_RATE},aformat=channel_layouts=stereo,volume={original_gain_db:.6}dB,apad,atrim=end_sample={sample_count}[original];\
                 [{envelope_index}:a:0]pan=stereo|c0=c0|c1=c0,asplit=2[inside][invert];\
                 [invert]aeval=exprs='1-val(0)|1-val(1)'[outside];\
                 [bed][inside]amultiply[separated];[original][outside]amultiply[ambient];\
                 [separated][ambient]amix=inputs=2:normalize=0[blended];"
            ));
            label = "blended";
        }
        if let Some((duck_index, _)) = duck_input {
            graph.push_str(&format!(
                "[{duck_index}:a:0]pan=stereo|c0=c0|c1=c0[duck];[{label}][duck]amultiply[background];"
            ));
            label = "background";
        }
        label
    });
    let filter = match (output_kind, voice, bed) {
        (MixOutput::Dialogue, Some(_), _) => format!(
            "{graph}[voice]atrim=end_sample={sample_count},aformat=sample_fmts=flt:channel_layouts=mono[out]"
        ),
        (MixOutput::Background, _, Some(bed)) => {
            format!("{graph}[{bed}]aformat=sample_fmts=flt:channel_layouts=stereo[out]")
        }
        (MixOutput::Combined, Some(_), Some(bed)) => format!(
            "{graph}[voice]pan=stereo|c0=c0|c1=c0[dialogue];[dialogue][{bed}]amix=inputs=2:duration=longest:normalize=0,atrim=end_sample={sample_count},aformat=sample_fmts=flt:channel_layouts=stereo[out]"
        ),
        (MixOutput::Combined, Some(_), None) => format!(
            "{graph}[voice]atrim=end_sample={sample_count},aformat=sample_fmts=flt:channel_layouts=mono[out]"
        ),
        _ => return Err("Mix artifact request does not match its sources".into()),
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
    if super::policy::wav_sample_count(output, if background_input.is_some() { 2 } else { 1 })?
        != sample_count
    {
        return Err("Mixed video dubbing audio has an invalid duration".into());
    }
    Ok(())
}
