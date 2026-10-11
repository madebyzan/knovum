#[cfg(test)]
#[path = "tests/artifact.rs"]
mod tests;
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::Path,
};

use knovum_inference_contracts::image_wire::{TensorDataType, TensorFileReference};
use sha2::{Digest, Sha256};

pub(crate) fn tensor_bytes(
    shape: [u32; 4],
    dtype: TensorDataType,
    maximum: u64,
) -> Result<u64, String> {
    let width = if dtype == TensorDataType::Uint8 { 1 } else { 4 };
    let size = shape
        .into_iter()
        .try_fold(width, |bytes: u64, dim| {
            if dim == 0 || dim > 16384 {
                None
            } else {
                bytes.checked_mul(u64::from(dim))
            }
        })
        .filter(|size| *size <= maximum);
    size.ok_or_else(|| {
        "invalid_input: tensor exceeds the admitted dimensions or byte budget".into()
    })
}

pub(crate) fn valid_name(name: &str, maximum: usize, dots: bool) -> bool {
    !name.is_empty()
        && name.len() <= maximum
        && name.as_bytes()[0].is_ascii_alphanumeric()
        && name.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || byte == b'-' || dots && matches!(byte, b'.' | b'_')
        })
}

pub(crate) fn open_regular(path: &Path) -> Result<File, String> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(windows_sys::Win32::Storage::FileSystem::FILE_FLAG_OPEN_REPARSE_POINT);
    }
    let file = options
        .open(path)
        .map_err(|error| format!("invalid_input: {error}"))?;
    let metadata = file.metadata().map_err(|error| error.to_string())?;
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes()
            & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
            != 0
        {
            return Err("invalid_input: artifact is a reparse point".into());
        }
    }
    if !metadata.is_file() {
        return Err("invalid_input: artifact is not a regular file".into());
    }
    Ok(file)
}

pub(crate) fn read_tensor(
    directory: &Path,
    reference: &TensorFileReference,
    maximum: u64,
) -> Result<Vec<u8>, String> {
    let size = tensor_bytes(reference.shape, reference.data_type, maximum)?;
    if reference.byte_length != size
        || !valid_name(&reference.file_name, 160, true)
        || reference.sha256.len() != 64
        || !reference
            .sha256
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("invalid_input: tensor reference is invalid".into());
    }
    let mut file = open_regular(&directory.join(&reference.file_name))?;
    if file.metadata().map_err(|error| error.to_string())?.len() != size {
        return Err("invalid_input: tensor file length differs".into());
    }
    let mut bytes = Vec::with_capacity(size as usize);
    Read::by_ref(&mut file)
        .take(size + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() as u64 != size || hex::encode(Sha256::digest(&bytes)) != reference.sha256 {
        return Err("invalid_input: tensor file content differs".into());
    }
    if reference.data_type == TensorDataType::Float32LittleEndian
        && bytes
            .chunks_exact(4)
            .any(|chunk| !f32::from_le_bytes(chunk.try_into().expect("float bytes")).is_finite())
    {
        return Err("invalid_input: tensor contains non-finite values".into());
    }
    Ok(bytes)
}

pub(crate) fn write_tensor(
    directory: &Path,
    name: &str,
    shape: [u32; 4],
    dtype: TensorDataType,
    bytes: &[u8],
) -> Result<TensorFileReference, String> {
    if !valid_name(name, 160, true)
        || tensor_bytes(shape, dtype, 256 * 1024 * 1024)? != bytes.len() as u64
    {
        return Err("invalid_output: invalid output tensor".into());
    }
    write_bytes(directory, name, bytes)?;
    Ok(TensorFileReference {
        file_name: name.into(),
        byte_length: bytes.len() as u64,
        sha256: hex::encode(Sha256::digest(bytes)),
        shape,
        data_type: dtype,
    })
}

pub(crate) fn write_bytes(directory: &Path, name: &str, bytes: &[u8]) -> Result<(), String> {
    if !valid_name(name, 160, true) || bytes.is_empty() || bytes.len() > 256 * 1024 * 1024 {
        return Err("invalid_output: artifact exceeds its file budget".into());
    }
    let target = directory.join(name);
    let temporary = directory.join(format!(".{name}.part"));
    let outcome = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|error| format!("resource_exhausted: {error}"))?;
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(|error| format!("resource_exhausted: {error}"))?;
        fs::hard_link(&temporary, &target)
            .map_err(|error| format!("resource_exhausted: {error}"))?;
        Ok(())
    })();
    let _ = fs::remove_file(temporary);
    outcome
}

pub(crate) fn require_directory(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| format!("invalid_input: {error}"))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("invalid_input: directory must be regular".into());
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes()
            & windows_sys::Win32::Storage::FileSystem::FILE_ATTRIBUTE_REPARSE_POINT
            != 0
        {
            return Err("invalid_input: directory cannot be a reparse point".into());
        }
    }
    Ok(())
}
