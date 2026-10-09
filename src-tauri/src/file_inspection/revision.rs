use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

const BUFFER_BYTES: usize = 256 * 1024;

pub fn content_revision(path: &Path) -> Result<String, String> {
    content_revision_cancellable(path, || Ok(()))
}

pub(crate) fn content_revision_cancellable<E>(
    path: &Path,
    mut check_cancelled: impl FnMut() -> Result<(), E>,
) -> Result<String, E>
where
    E: From<String>,
{
    check_cancelled()?;
    let metadata = fs::symlink_metadata(path).map_err(|error| E::from(error.to_string()))?;
    if metadata.file_type().is_symlink() {
        return Err(E::from("Symbolic links are not valid Space sources".into()));
    }
    if !metadata.is_file() {
        return Err(E::from("Only regular files have Space revisions".into()));
    }
    let digest = file_digest(path, &mut check_cancelled)?;
    Ok(format!("sha256:{digest}", digest = hex::encode(digest)))
}

pub fn verify_content_revision(path: &Path, expected: &str) -> Result<(), String> {
    if content_revision(path)? != expected {
        return Err("Resource content revision changed during processing".into());
    }
    Ok(())
}

fn file_digest<E>(
    path: &Path,
    check_cancelled: &mut impl FnMut() -> Result<(), E>,
) -> Result<sha2::digest::Output<Sha256>, E>
where
    E: From<String>,
{
    let mut file = File::open(path).map_err(|error| E::from(error.to_string()))?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; BUFFER_BYTES];
    loop {
        check_cancelled()?;
        let read = file
            .read(&mut buffer)
            .map_err(|error| E::from(error.to_string()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize())
}

#[cfg(test)]
#[path = "tests/revision.rs"]
mod tests;
