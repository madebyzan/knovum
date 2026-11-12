use std::{
    fs,
    io::{Read, Write},
    path::Path,
    time::{Duration, Instant},
};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::{
    admission::Priority,
    device_profiles,
    speech::{self, co_located_model_directory, Execution},
};
use crate::{
    ai::contracts::{
        runtime::RuntimeDeviceProfile,
        speech_wire::{
            InferenceRequestIdentity, SpeechSeparateParams, SpeechSeparation,
            SynthesizedAudioFileReference, SEPARATION_MAX_DURATION_MS,
            SEPARATION_SAMPLE_RATE as SAMPLE_RATE,
        },
    },
    local_resources::{model_catalog::LocalModelId, preparation::PreparedModels},
};

const CHANNELS: u16 = 2;
const MAX_SAMPLE_COUNT: u64 = SAMPLE_RATE as u64 * (SEPARATION_MAX_DURATION_MS / 1_000);
const MAX_STEM_BYTES: u64 = MAX_SAMPLE_COUNT * CHANNELS as u64 * 2 + 65_536;

pub(crate) struct SeparatedAudio {
    pub(crate) vocals: SeparatedAudioMetadata,
    pub(crate) background: SeparatedAudioMetadata,
}

pub(crate) struct SeparatedAudioMetadata {
    pub(crate) sha256: String,
    pub(crate) sample_rate: u32,
    pub(crate) sample_count: u64,
}

#[derive(Clone, Copy)]
pub(crate) struct SeparationTargets<'a> {
    pub(crate) vocals: &'a Path,
    pub(crate) background: &'a Path,
}

pub(crate) async fn separate_to_paths<R: Runtime>(
    app: &AppHandle<R>,
    resources: &PreparedModels,
    device_profile: RuntimeDeviceProfile,
    params: SpeechSeparateParams,
    cancellation: &CancellationToken,
    priority: Priority,
    targets: SeparationTargets<'_>,
) -> Result<SeparatedAudio, String> {
    let model = LocalModelId::from_separation_spec(&params.identity.model_spec_id)?;
    if device_profile.execution_profile_id
        != device_profiles::separation::execution_id(model, &device_profile.device)?
    {
        return Err("runtime_mismatch: source separation model and device profile differ".into());
    }
    let snapshot = model
        .snapshot()
        .ok_or("Source separation model snapshot is unavailable")?;
    let model_path = co_located_model_directory(resources, snapshot.models())?;
    speech::execute(
        Execution {
            app,
            model_path,
            device_profile,
            cancellation,
            priority,
        },
        params,
        None,
        None,
        |separation, work_directory, identity, _| {
            finish(separation, work_directory, identity, cancellation, targets)
        },
    )
    .await
}

fn finish(
    separation: SpeechSeparation,
    work_directory: &Path,
    identity: &InferenceRequestIdentity,
    cancellation: &CancellationToken,
    targets: SeparationTargets<'_>,
) -> Result<SeparatedAudio, String> {
    if separation.request_id != identity.request_id
        || separation.attempt_epoch != identity.attempt_epoch
        || separation.model_spec_id != identity.model_spec_id
    {
        return Err("invalid_output: source separation response identity differs".into());
    }
    validate_reference(&separation.vocals_ref)?;
    validate_reference(&separation.background_ref)?;
    if separation.sample_count == 0 || separation.sample_count > MAX_SAMPLE_COUNT {
        return Err("invalid_output: source separation returned an invalid timeline".into());
    }
    // Publishing the stems is the last stretch of the same request, so it continues under the
    // same budget, started here because the worker has only just handed the artifacts over.
    let deadline = Instant::now() + Duration::from_millis(identity.budget_ms);
    let vocals = copy_artifact(
        work_directory,
        &identity.request_id,
        &separation.vocals_ref,
        separation.sample_count,
        deadline,
        cancellation,
        targets.vocals,
    )?;
    let background = match copy_artifact(
        work_directory,
        &identity.request_id,
        &separation.background_ref,
        separation.sample_count,
        deadline,
        cancellation,
        targets.background,
    ) {
        Ok(value) => value,
        Err(error) => {
            let _ = fs::remove_file(targets.vocals);
            return Err(error);
        }
    };
    if let Err(error) =
        fs::remove_dir(work_directory.join(format!("attempt-{}", identity.request_id)))
    {
        let _ = fs::remove_file(targets.vocals);
        let _ = fs::remove_file(targets.background);
        return Err(format!("invalid_output: {error}"));
    }
    if vocals.sample_count != background.sample_count {
        let _ = fs::remove_file(targets.vocals);
        let _ = fs::remove_file(targets.background);
        return Err("invalid_output: published separation stems have different timelines".into());
    }
    Ok(SeparatedAudio { vocals, background })
}

fn validate_reference(reference: &SynthesizedAudioFileReference) -> Result<(), String> {
    if reference.media_type != "audio/wav"
        || reference.sample_rate != SAMPLE_RATE
        || reference.channels != CHANNELS
        || reference.sample_format != "pcm_s16le"
        || reference.byte_length > MAX_STEM_BYTES
    {
        return Err("invalid_output: source separation stem metadata differs".into());
    }
    Ok(())
}

fn copy_artifact(
    work_directory: &Path,
    request_id: &str,
    reference: &SynthesizedAudioFileReference,
    sample_count: u64,
    deadline: Instant,
    cancellation: &CancellationToken,
    target: &Path,
) -> Result<SeparatedAudioMetadata, String> {
    let source =
        super::artifact::attempt_artifact_path(work_directory, request_id, &reference.file_name)?;
    let mut input = super::artifact::open_regular(&source)?;
    if input.metadata().map_err(|error| error.to_string())?.len() != reference.byte_length {
        return Err("invalid_output: source separation artifact length differs".into());
    }
    let parent = target
        .parent()
        .ok_or("Source separation staging path has no parent directory")?;
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let mut output = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(target)
        .map_err(|error| error.to_string())?;
    let copied = (|| {
        let mut digest = Sha256::new();
        let mut written = 0_u64;
        let mut buffer = [0_u8; 1024 * 1024];
        loop {
            check_active(cancellation, deadline)?;
            let read = input.read(&mut buffer).map_err(|error| error.to_string())?;
            if read == 0 {
                break;
            }
            written = written
                .checked_add(read as u64)
                .ok_or("resource_exhausted: source separation artifact size overflow")?;
            if written > reference.byte_length {
                return Err("invalid_output: source separation artifact grew while reading".into());
            }
            output
                .write_all(&buffer[..read])
                .map_err(|error| error.to_string())?;
            digest.update(&buffer[..read]);
        }
        output.sync_all().map_err(|error| error.to_string())?;
        let sha256 = hex::encode(digest.finalize());
        if written != reference.byte_length || sha256 != reference.sha256 {
            return Err("invalid_output: source separation artifact digest differs".into());
        }
        validate_wav(target, sample_count)?;
        Ok(SeparatedAudioMetadata {
            sha256,
            sample_rate: SAMPLE_RATE,
            sample_count,
        })
    })();
    let metadata = match copied {
        Ok(metadata) => metadata,
        Err(error) => {
            let _ = fs::remove_file(target);
            return Err(error);
        }
    };
    drop(input);
    drop(output);
    if let Err(error) = fs::remove_file(source) {
        let _ = fs::remove_file(target);
        return Err(format!("invalid_output: {error}"));
    }
    Ok(metadata)
}

fn check_active(cancellation: &CancellationToken, deadline: Instant) -> Result<(), String> {
    if cancellation.is_cancelled() {
        return Err("cancelled".into());
    }
    if Instant::now() >= deadline {
        return Err("deadline_exceeded".into());
    }
    Ok(())
}

fn validate_wav(path: &Path, sample_count: u64) -> Result<(), String> {
    let mut file = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut header = [0_u8; 44];
    file.read_exact(&mut header)
        .map_err(|_| "invalid_output: source separation WAV header is truncated".to_string())?;
    let data_bytes = sample_count
        .checked_mul(u64::from(CHANNELS))
        .and_then(|value| value.checked_mul(2))
        .ok_or("invalid_output: source separation WAV size overflow")?;
    let expected = data_bytes
        .checked_add(44)
        .ok_or("invalid_output: source separation WAV size overflow")?;
    if header[0..4] != *b"RIFF"
        || header[8..12] != *b"WAVE"
        || header[12..16] != *b"fmt "
        || u32::from_le_bytes([header[16], header[17], header[18], header[19]]) != 16
        || u16::from_le_bytes([header[20], header[21]]) != 1
        || u16::from_le_bytes([header[22], header[23]]) != CHANNELS
        || u32::from_le_bytes([header[24], header[25], header[26], header[27]]) != SAMPLE_RATE
        || u16::from_le_bytes([header[34], header[35]]) != 16
        || header[36..40] != *b"data"
        || u64::from(u32::from_le_bytes([
            header[40], header[41], header[42], header[43],
        ])) != data_bytes
        || fs::metadata(path).map_err(|error| error.to_string())?.len() != expected
    {
        return Err("invalid_output: source separation WAV metadata differs".into());
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/separation.rs"]
mod tests;
