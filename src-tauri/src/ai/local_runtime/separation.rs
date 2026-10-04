use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::{
    admission::Priority,
    speech::{self, Execution},
};
use crate::{
    ai::contracts::{
        runtime::RuntimeDeviceProfile,
        speech_wire::{
            InferenceRequestIdentity, SpeechSeparateParams, SpeechSeparateResult,
            SynthesizedAudioFileReference,
        },
    },
    local_resources::{
        models::catalog::{self, ModelKind},
        preparation::PreparedModels,
    },
};

const SAMPLE_RATE: u32 = 44_100;
const CHANNELS: u16 = 2;
const MAX_DURATION_SECONDS: u64 = 10 * 60;
const MAX_STEM_BYTES: u64 =
    SAMPLE_RATE as u64 * CHANNELS as u64 * 2 * MAX_DURATION_SECONDS + 65_536;

pub(crate) struct SeparatedAudioMetadata {
    pub(crate) sha256: String,
    pub(crate) sample_rate: u32,
    pub(crate) sample_count: u64,
}

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
) -> Result<SeparatedAudioMetadata, String> {
    if device_profile.device != "cpu" {
        return Err("unsupported_device: Spleeter source separation requires CPU".into());
    }
    let model_path = co_located_model_directory(resources)?;
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
        |result, work_directory, identity, _| {
            finish(result, work_directory, identity, cancellation, targets)
        },
    )
    .await
}

fn finish(
    result: SpeechSeparateResult,
    work_directory: &Path,
    identity: &InferenceRequestIdentity,
    cancellation: &CancellationToken,
    targets: SeparationTargets<'_>,
) -> Result<SeparatedAudioMetadata, String> {
    if result.request_id != identity.request_id
        || result.attempt_epoch != identity.attempt_epoch
        || result.model_spec_id != identity.model_spec_id
    {
        return Err("invalid_output: source separation response identity differs".into());
    }
    validate_reference(&result.vocals_ref)?;
    validate_reference(&result.background_ref)?;
    if result.sample_count == 0 || result.sample_count > SAMPLE_RATE as u64 * MAX_DURATION_SECONDS {
        return Err("invalid_output: source separation returned an invalid timeline".into());
    }
    let vocals = copy_artifact(
        work_directory,
        &identity.request_id,
        &result.vocals_ref,
        result.sample_count,
        identity.deadline_ms,
        cancellation,
        targets.vocals,
    )?;
    let background = match copy_artifact(
        work_directory,
        &identity.request_id,
        &result.background_ref,
        result.sample_count,
        identity.deadline_ms,
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
    Ok(background)
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
    deadline_ms: u64,
    cancellation: &CancellationToken,
    target: &Path,
) -> Result<SeparatedAudioMetadata, String> {
    let source = artifact_path(work_directory, request_id, &reference.file_name)?;
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
            check_active(cancellation, deadline_ms)?;
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
        let sha256 = format!("{:x}", digest.finalize());
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

fn check_active(cancellation: &CancellationToken, deadline_ms: u64) -> Result<(), String> {
    if cancellation.is_cancelled() {
        return Err("cancelled".into());
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis();
    if now >= u128::from(deadline_ms) {
        return Err("deadline_exceeded".into());
    }
    Ok(())
}

fn artifact_path(
    work_directory: &Path,
    request_id: &str,
    file_name: &str,
) -> Result<PathBuf, String> {
    let relative = Path::new(file_name);
    let attempt_name = format!("attempt-{request_id}");
    let mut components = relative.components();
    if !matches!(components.next(), Some(std::path::Component::Normal(name)) if name.to_str() == Some(attempt_name.as_str()))
        || !matches!(components.next(), Some(std::path::Component::Normal(_)))
        || components.next().is_some()
        || !file_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'/'))
    {
        return Err("invalid_output: source separation artifact name is invalid".into());
    }
    let directory = work_directory.join(&attempt_name);
    super::artifact::checked_directory(&directory)?;
    Ok(work_directory.join(relative))
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

fn co_located_model_directory(resources: &PreparedModels) -> Result<&Path, String> {
    let kinds = catalog::SPEECH_SEPARATION_MODELS;
    let anchor = resources.path(kinds[0])?;
    let directory = anchor
        .parent()
        .ok_or("runtime_mismatch: Spleeter model directory is missing")?;
    for kind in kinds {
        let expected = directory.join(catalog::descriptor(*kind).file);
        if resources.path(*kind)? != expected {
            return Err(format!(
                "runtime_mismatch: Spleeter model part {} is not co-located",
                match kind {
                    ModelKind::SpleeterAccompaniment => "accompaniment",
                    ModelKind::SpleeterVocals => "vocals",
                    _ => "unknown",
                }
            ));
        }
    }
    Ok(directory)
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    #[test]
    fn source_separation_artifact_copy_validates_timeline_and_cleans_worker_file() {
        let request_id = uuid::Uuid::new_v4().to_string();
        let root = std::env::temp_dir().join(format!("knovum-runtime-separation-{request_id}"));
        let attempt = root.join(format!("attempt-{request_id}"));
        fs::create_dir_all(&attempt).unwrap();
        let source = attempt.join("background.wav");
        write_wav(&source, 4);
        let payload = fs::read(&source).unwrap();
        let reference = SynthesizedAudioFileReference {
            file_name: format!("attempt-{request_id}/background.wav"),
            media_type: "audio/wav".into(),
            byte_length: payload.len() as u64,
            sha256: format!("{:x}", Sha256::digest(&payload)),
            sample_rate: SAMPLE_RATE,
            channels: CHANNELS,
            sample_format: "pcm_s16le".into(),
        };
        let target = root.join("published.wav");
        let metadata = copy_artifact(
            &root,
            &request_id,
            &reference,
            4,
            u64::MAX,
            &CancellationToken::new(),
            &target,
        )
        .unwrap();
        assert_eq!(metadata.sample_count, 4);
        assert!(target.is_file());
        assert!(!source.exists());

        write_wav(&source, 4);
        let invalid_target = root.join("invalid.wav");
        let error = copy_artifact(
            &root,
            &request_id,
            &reference,
            5,
            u64::MAX,
            &CancellationToken::new(),
            &invalid_target,
        )
        .err()
        .unwrap();
        assert!(error.contains("WAV metadata differs"));
        assert!(!invalid_target.exists());
        fs::remove_dir_all(root).unwrap();
    }

    fn write_wav(path: &Path, sample_count: u64) {
        let data_bytes = sample_count * u64::from(CHANNELS) * 2;
        let mut header = [0_u8; 44];
        header[0..4].copy_from_slice(b"RIFF");
        header[4..8].copy_from_slice(&u32::try_from(data_bytes + 36).unwrap().to_le_bytes());
        header[8..12].copy_from_slice(b"WAVE");
        header[12..16].copy_from_slice(b"fmt ");
        header[16..20].copy_from_slice(&16_u32.to_le_bytes());
        header[20..22].copy_from_slice(&1_u16.to_le_bytes());
        header[22..24].copy_from_slice(&CHANNELS.to_le_bytes());
        header[24..28].copy_from_slice(&SAMPLE_RATE.to_le_bytes());
        header[28..32].copy_from_slice(&(SAMPLE_RATE * u32::from(CHANNELS) * 2).to_le_bytes());
        header[32..34].copy_from_slice(&(CHANNELS * 2).to_le_bytes());
        header[34..36].copy_from_slice(&16_u16.to_le_bytes());
        header[36..40].copy_from_slice(b"data");
        header[40..44].copy_from_slice(&u32::try_from(data_bytes).unwrap().to_le_bytes());
        let mut file = fs::File::create(path).unwrap();
        file.write_all(&header).unwrap();
        file.write_all(&vec![0; data_bytes as usize]).unwrap();
    }
}
