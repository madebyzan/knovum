pub(super) use render_commit::commit_render;
mod render_commit;
pub use recovery::reconcile_interrupted;
mod recovery;
#[cfg(test)]
mod tests;
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, Runtime};

use super::contracts::{SpeechAudioOutput, SpeechSegmentRenderOutput};
use crate::spaces::assets;

pub fn write_preview<R: Runtime>(
    app: &AppHandle<R>,
    job_id: &str,
    audio: crate::ai::local_runtime::speech::SynthesizedAudio,
) -> Result<SpeechAudioOutput, String> {
    validate_audio_payload(&audio)?;
    let directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("speech-previews");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let path = directory.join(format!("{job_id}.wav"));
    atomic_write(&path, &audio.bytes)?;
    Ok(SpeechAudioOutput {
        path: path.to_string_lossy().into_owned(),
        sha256: audio.sha256,
        sample_rate: audio.sample_rate,
        sample_count: audio.sample_count,
    })
}

pub fn segment_path<R: Runtime>(
    app: &AppHandle<R>,
    project_id: &str,
    revision_id: &str,
    segment_id: &str,
    input_digest: &str,
    attempt_id: &str,
) -> Result<PathBuf, String> {
    for (value, label) in [
        (project_id, "project"),
        (revision_id, "revision"),
        (segment_id, "segment"),
        (attempt_id, "attempt"),
    ] {
        uuid::Uuid::parse_str(value).map_err(|_| format!("Speech {label} identity is invalid"))?;
    }
    if input_digest.len() != 64 || !input_digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("Speech input digest is invalid".to_owned());
    }
    Ok(assets::asset_directory(app)?
        .join("speech")
        .join(project_id)
        .join(revision_id)
        .join(format!("{segment_id}-{input_digest}-{attempt_id}.wav")))
}

pub fn publish_segment(
    path: &Path,
    audio: &crate::ai::local_runtime::speech::SynthesizedAudio,
) -> Result<PathBuf, String> {
    validate_audio_payload(audio)?;
    let directory = path
        .parent()
        .ok_or("Speech output path has no parent directory")?;
    fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    atomic_write(path, &audio.bytes)?;
    let actual = hash_file(path)?;
    if actual != audio.sha256 {
        return Err(cleanup_failed_output(
            path,
            "invalid_output: published speech hash differs".to_owned(),
        ));
    }
    Ok(path.to_owned())
}

pub(super) fn validate_synthesized_audio(
    audio: &crate::ai::local_runtime::speech::SynthesizedAudio,
) -> Result<(), String> {
    validate_audio_payload(audio)
}

pub fn validate_cached<R: Runtime>(
    app: &AppHandle<R>,
    mut output: SpeechSegmentRenderOutput,
) -> Result<SpeechSegmentRenderOutput, String> {
    let path = assets::authorize_spaces_asset(app, Path::new(&output.path))?;
    if hash_file(&path)? != output.sha256 {
        return Err("Speech render cache is corrupt".to_owned());
    }
    output.path = path.to_string_lossy().into_owned();
    Ok(output)
}

pub fn discard_preview<R: Runtime>(app: &AppHandle<R>, candidate: &str) -> Result<(), String> {
    let directory = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("speech-previews");
    let root = directory
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let candidate = Path::new(candidate);
    let parent = candidate
        .parent()
        .ok_or("Speech preview has no parent directory")?
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if parent != root || candidate.extension().and_then(|value| value.to_str()) != Some("wav") {
        return Err("Speech preview path is outside the managed cache".to_owned());
    }
    let path = match candidate.canonicalize() {
        Ok(path) => path,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.to_string()),
    };
    if path.parent() != Some(root.as_path()) {
        return Err("Speech preview resolves outside the managed cache".to_owned());
    }
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if path.exists() {
        return Err("Speech output path is already occupied".to_owned());
    }
    let part = path.with_extension("wav.part");
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&part)
        .map_err(|error| error.to_string())?;
    let publication = (|| {
        file.write_all(bytes).map_err(|error| error.to_string())?;
        file.sync_all().map_err(|error| error.to_string())?;
        fs::rename(&part, path).map_err(|error| error.to_string())
    })();
    drop(file);
    publication.map_err(|cause| cleanup_failed_output(&part, cause))
}

fn cleanup_failed_output(path: &Path, mut cause: String) -> String {
    if let Err(error) = fs::remove_file(path) {
        if error.kind() != std::io::ErrorKind::NotFound {
            cause.push_str(&format!("; failed to remove {}: {error}", path.display()));
        }
    }
    cause
}

pub(super) fn hash_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(hex::encode(digest.finalize()))
}

fn validate_audio_payload(
    audio: &crate::ai::local_runtime::speech::SynthesizedAudio,
) -> Result<(), String> {
    let metadata = crate::media::wav::inspect(std::io::Cursor::new(&audio.bytes))
        .map_err(|cause| format!("invalid_output: {cause}"))?;
    let expected = super::audio_format::WavPrecision::Float32.spec(audio.sample_rate, 1);
    if audio.sample_count == 0
        || metadata.spec != expected
        || metadata.sample_count != audio.sample_count
    {
        return Err("invalid_output: WAV audio metadata differs".into());
    }
    Ok(())
}
