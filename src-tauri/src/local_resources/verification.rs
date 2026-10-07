use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct VerificationReceipt {
    schema_version: u8,
    sha256: String,
    size: u64,
    modified_nanos: u64,
}

pub(crate) fn receipt_matches(
    path: &Path,
    sha256: &str,
    metadata: &fs::Metadata,
) -> Result<bool, String> {
    let bytes = match fs::read(receipt_path(path)) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.to_string()),
    };
    let receipt: VerificationReceipt =
        serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    Ok(receipt.schema_version == 1
        && receipt.sha256 == sha256
        && receipt.size == metadata.len()
        && receipt.modified_nanos == modified_nanos(metadata)?)
}

pub(crate) fn verify_and_record_cancellable(
    path: &Path,
    sha256: &str,
    cancellation: &CancellationToken,
    cancellation_message: &str,
) -> Result<bool, String> {
    if !verify_cancellable(path, sha256, cancellation, cancellation_message)? {
        return Ok(false);
    }
    record(path, sha256)?;
    Ok(true)
}

pub(crate) fn verify_cancellable(
    path: &Path,
    sha256: &str,
    cancellation: &CancellationToken,
    cancellation_message: &str,
) -> Result<bool, String> {
    Ok(sha256_file(path, cancellation, cancellation_message)? == sha256)
}

pub(crate) fn record(path: &Path, sha256: &str) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|error| error.to_string())?;
    let receipt = VerificationReceipt {
        schema_version: 1,
        sha256: sha256.to_owned(),
        size: metadata.len(),
        modified_nanos: modified_nanos(&metadata)?,
    };
    let bytes = serde_json::to_vec(&receipt).map_err(|error| error.to_string())?;
    let target = receipt_path(path);
    let temporary = target.with_extension("json.tmp");
    fs::write(&temporary, bytes).map_err(|error| error.to_string())?;
    if target.exists() {
        fs::remove_file(&target).map_err(|error| error.to_string())?;
    }
    fs::rename(temporary, target).map_err(|error| error.to_string())
}

fn receipt_path(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .expect("managed local-resource paths always contain a file name")
        .to_os_string();
    name.push(".verified.json");
    path.with_file_name(name)
}

fn modified_nanos(metadata: &fs::Metadata) -> Result<u64, String> {
    metadata
        .modified()
        .map_err(|error| error.to_string())?
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos()
        .try_into()
        .map_err(|error: std::num::TryFromIntError| error.to_string())
}

fn sha256_file(
    path: &Path,
    cancellation: &CancellationToken,
    cancellation_message: &str,
) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        if cancellation.is_cancelled() {
            return Err(cancellation_message.to_string());
        }
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
    }
    Ok(hex::encode(hash.finalize()))
}
