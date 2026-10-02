use std::{fs, io::Read, path::Path};

use sha2::{Digest, Sha256};

use crate::ai::capabilities::agent_artifacts;
use crate::app_error::{AppError, AppResult};

pub(super) fn read(root: &Path, name: &str, limit: u64) -> AppResult<Vec<u8>> {
    let invalid = |message| AppError::new("invalid_response", message);
    let path = agent_artifacts::resolve(root, name).map_err(|e| {
        invalid(format!(
            "Desktop Agent did not submit a valid task result: {e}"
        ))
    })?;
    let file = fs::File::open(&path).map_err(|e| invalid(e.to_string()))?;
    let metadata = file.metadata().map_err(|e| invalid(e.to_string()))?;
    if !metadata.is_file() || metadata.len() == 0 || metadata.len() > limit {
        return Err(invalid(
            "Desktop Agent result is not a nonempty regular file within its task byte budget"
                .into(),
        ));
    }
    let revision = crate::file_inspection::content_revision(&path).map_err(&invalid)?;
    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| invalid(e.to_string()))?;
    if bytes.is_empty() || bytes.len() as u64 > limit {
        return Err(invalid(
            "Desktop Agent result is empty or exceeds its task byte budget".into(),
        ));
    }
    if format!("sha256:{:x}", Sha256::digest(&bytes)) != revision {
        return Err(invalid("Desktop Agent result changed while reading".into()));
    }
    if fs::metadata(&path)
        .map_err(|e| invalid(e.to_string()))?
        .len()
        != bytes.len() as u64
    {
        return Err(invalid(
            "Desktop Agent result size changed while reading".into(),
        ));
    }
    crate::file_inspection::verify_content_revision(&path, &revision).map_err(invalid)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests;
