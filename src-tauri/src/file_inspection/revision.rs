use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::Read,
    path::Path,
};

const BUFFER_BYTES: usize = 256 * 1024;

pub fn content_revision(path: &Path) -> Result<String, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() {
        return Err("Symbolic links are not valid Space sources".into());
    }
    if !metadata.is_file() {
        return Err("Only regular files have Space revisions".into());
    }
    let digest = file_digest(path)?;
    Ok(format!("sha256:{digest:x}"))
}

fn file_digest(path: &Path) -> Result<sha2::digest::Output<Sha256>, String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0_u8; BUFFER_BYTES];
    loop {
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize())
}
