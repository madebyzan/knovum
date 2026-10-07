use std::{fs, io::Read, path::Path};

use sha2::{Digest, Sha256};

pub(super) fn read_artifact(
    work_directory: &Path,
    request_id: &str,
    file_name: &str,
    byte_length: u64,
    expected_sha256: &str,
    maximum_bytes: u64,
    cleanup_attempt: bool,
) -> Result<Vec<u8>, String> {
    if byte_length == 0 || byte_length > maximum_bytes {
        return Err("resource_exhausted: worker artifact exceeds its byte limit".into());
    }
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
        return Err("invalid_output: worker artifact name is invalid".into());
    }
    let attempt_directory = work_directory.join(&attempt_name);
    super::super::artifact::checked_directory(&attempt_directory)?;
    let path = work_directory.join(relative);
    let mut file = super::super::artifact::open_regular(&path)?;
    let metadata = file
        .metadata()
        .map_err(|error| format!("invalid_output: {error}"))?;
    if !metadata.file_type().is_file() || metadata.len() != byte_length {
        return Err("invalid_output: worker artifact metadata differs".into());
    }
    let mut bytes = Vec::with_capacity(byte_length as usize);
    file.by_ref()
        .take(byte_length + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("invalid_output: {error}"))?;
    if bytes.len() as u64 != byte_length {
        return Err("invalid_output: worker artifact changed while reading".into());
    }
    let digest = hex::encode(Sha256::digest(&bytes));
    if digest != expected_sha256 {
        return Err("invalid_output: worker artifact digest differs".into());
    }
    drop(file);
    fs::remove_file(path).map_err(|error| format!("invalid_output: {error}"))?;
    if cleanup_attempt {
        fs::remove_dir(attempt_directory).map_err(|error| format!("invalid_output: {error}"))?;
    }
    Ok(bytes)
}
