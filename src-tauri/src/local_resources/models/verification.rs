use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::ModelDescriptor;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
struct VerificationReceipt {
    schema_version: u8,
    sha256: String,
    size: u64,
    modified_nanos: u64,
}

pub(super) fn receipt_matches(
    path: &Path,
    descriptor: ModelDescriptor,
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
        && receipt.sha256 == descriptor.sha256
        && receipt.size == metadata.len()
        && receipt.modified_nanos == modified_nanos(metadata)?)
}

pub(super) fn verify_and_record(path: &Path, descriptor: ModelDescriptor) -> Result<bool, String> {
    if !verify(path, descriptor)? {
        return Ok(false);
    }
    record(path, descriptor)?;
    Ok(true)
}

pub(super) fn verify(path: &Path, descriptor: ModelDescriptor) -> Result<bool, String> {
    Ok(sha256_file(path)? == descriptor.sha256)
}

pub(super) fn record(path: &Path, descriptor: ModelDescriptor) -> Result<(), String> {
    let metadata = fs::metadata(path).map_err(|error| error.to_string())?;
    let receipt = VerificationReceipt {
        schema_version: 1,
        sha256: descriptor.sha256.to_owned(),
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

pub(super) fn remove_receipt(path: &Path) -> Result<(), String> {
    match fs::remove_file(receipt_path(path)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

fn receipt_path(path: &Path) -> PathBuf {
    let mut name = path
        .file_name()
        .expect("local model paths always contain a file name")
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

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hash.finalize()))
}
