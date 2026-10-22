use crate::artifact;
use knovum_inference_contracts::speech_wire::SpeechFileReference;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(crate) fn attempt(work: &Path, request: &str) -> Result<PathBuf, String> {
    let id = uuid::Uuid::parse_str(request)
        .map_err(|_| "invalid_input: speech request must be a UUID")?;
    if id.to_string() != request {
        return Err("invalid_input: speech request UUID must be canonical".into());
    }
    let path = work.join(format!("attempt-{request}"));
    // The attempt directory is private to this user; Windows inherits the work directory's ACL.
    let builder = fs::DirBuilder::new();
    #[cfg(unix)]
    let builder = {
        use std::os::unix::fs::DirBuilderExt;
        let mut builder = builder;
        builder.mode(0o700);
        builder
    };
    builder
        .create(&path)
        .map_err(|error| format!("resource_exhausted: {error}"))?;
    Ok(path)
}

pub(crate) fn json(
    work: &Path,
    request: &str,
    value: &impl serde::Serialize,
) -> Result<SpeechFileReference, String> {
    let bytes = serde_json::to_vec(value).map_err(|error| format!("invalid_output: {error}"))?;
    if bytes.len() > 64 * 1024 * 1024 {
        return Err("resource_exhausted: transcript exceeds its artifact budget".into());
    }
    let directory = attempt(work, request)?;
    artifact::write_bytes(&directory, "transcript.json", &bytes)?;
    Ok(SpeechFileReference {
        file_name: format!("attempt-{request}/transcript.json"),
        media_type: "application/vnd.knovum.speech-transcript+json".into(),
        byte_length: bytes.len() as u64,
        sha256: hex::encode(Sha256::digest(&bytes)),
    })
}
