use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;

use crate::file_inspection::modified_millis;
use crate::spaces::assets::enrichment::ensure_active;

const BUFFER_BYTES: usize = 256 * 1024;

pub struct CopiedResource {
    pub revision: String,
    pub size: u64,
    pub modified_at: u64,
}

pub fn copy_resource(
    source: &Path,
    target: &Path,
    cancellation: &CancellationToken,
    mut on_bytes: impl FnMut(u64) -> Result<(), String>,
) -> Result<CopiedResource, String> {
    let metadata = fs::symlink_metadata(source).map_err(|error| error.to_string())?;
    reject_source(&metadata)?;
    let (digest, size) = copy_file(source, target, cancellation, &mut on_bytes)?;
    Ok(CopiedResource {
        revision: format!("sha256:{digest:x}"),
        size,
        modified_at: modified_millis(&metadata)?,
    })
}

fn copy_file(
    source: &Path,
    target: &Path,
    cancellation: &CancellationToken,
    on_bytes: &mut impl FnMut(u64) -> Result<(), String>,
) -> Result<(sha2::digest::Output<Sha256>, u64), String> {
    let mut input = File::open(source).map_err(|error| error.to_string())?;
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(target)
        .map_err(|error| error.to_string())?;
    let mut digest = Sha256::new();
    let mut buffer = vec![0_u8; BUFFER_BYTES];
    let mut size = 0_u64;
    loop {
        ensure_active(cancellation)?;
        let read = input.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        output
            .write_all(&buffer[..read])
            .map_err(|error| error.to_string())?;
        digest.update(&buffer[..read]);
        size += read as u64;
        on_bytes(read as u64)?;
    }
    output.flush().map_err(|error| error.to_string())?;
    Ok((digest.finalize(), size))
}

fn reject_source(metadata: &fs::Metadata) -> Result<(), String> {
    if metadata.file_type().is_symlink() {
        return Err("A symbolic link cannot be imported as a Space resource".to_owned());
    }
    if !metadata.is_file() {
        return Err("Only regular files can be imported into the Space".to_owned());
    }
    Ok(())
}

#[cfg(test)]
mod tests;
