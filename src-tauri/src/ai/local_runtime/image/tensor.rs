use sha2::{Digest, Sha256};
use std::io::Read;
use tokio::io::AsyncWriteExt;

pub(super) fn read_mask(
    path: &std::path::Path,
    expected_bytes: u64,
    expected_sha256: &str,
) -> Result<Vec<f32>, String> {
    let mut file = crate::ai::local_runtime::artifact::open_regular(path)?;
    if file
        .metadata()
        .map_err(|error| format!("invalid_output: {error}"))?
        .len()
        != expected_bytes
    {
        return Err("invalid_output: image segmentation mask file is invalid".into());
    }
    let mut bytes = Vec::with_capacity(expected_bytes as usize);
    file.read_to_end(&mut bytes)
        .map_err(|error| format!("invalid_output: {error}"))?;
    if bytes.len() != expected_bytes as usize
        || hex::encode(Sha256::digest(&bytes)) != expected_sha256
    {
        return Err("invalid_output: image segmentation mask digest differs".into());
    }
    let mask = bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect::<Vec<_>>();
    if mask.iter().any(|value| !value.is_finite()) {
        return Err("invalid_output: image segmentation mask contains non-finite values".into());
    }
    Ok(mask)
}

pub(super) async fn stage_tensor(path: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
    let part = path.with_extension("f32.part");
    let mut file = tokio::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&part)
        .await
        .map_err(|error| format!("resource_exhausted: {error}"))?;
    let tensor_write = async {
        file.write_all(bytes).await?;
        file.sync_all().await?;
        drop(file);
        tokio::fs::rename(&part, path).await
    }
    .await;
    if let Err(error) = tensor_write {
        let _ = tokio::fs::remove_file(&part).await;
        return Err(format!("resource_exhausted: {error}"));
    }
    Ok(())
}

pub(super) fn validate_input(input: &[f32]) -> Result<(), String> {
    if input.is_empty() || !input.len().is_multiple_of(3) || input.iter().any(|value| !value.is_finite()) {
        return Err("invalid_input: image segmentation tensor is invalid".into());
    }
    let pixels = input.len() / 3;
    let edge = (pixels as f64).sqrt().round() as usize;
    if edge * edge != pixels {
        return Err("invalid_input: image segmentation tensor must be square".into());
    }
    Ok(())
}

pub(super) fn encode_f32(values: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(std::mem::size_of_val(values));
    for value in values {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}

pub(super) fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
