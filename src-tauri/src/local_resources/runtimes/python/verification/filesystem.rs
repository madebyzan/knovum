use std::{
    collections::HashMap,
    fs,
    path::{Component, Path, PathBuf},
};

use tokio_util::sync::CancellationToken;

use super::super::RuntimeError;
use super::{RuntimeFile, MANIFEST_NAME};

const MAX_RUNTIME_FILE_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const MAX_RUNTIME_TOTAL_BYTES: u64 = 16 * 1024 * 1024 * 1024;
const MAX_RUNTIME_FILES: usize = 100_000;

pub(super) fn verify(
    root: &Path,
    files: &[RuntimeFile],
    executable: &str,
    package_root: &str,
    cancellation: &CancellationToken,
) -> Result<(PathBuf, PathBuf), RuntimeError> {
    let declared = verify_declared_files(root, files, cancellation)?;
    let executable = required_file(root, executable, &declared)?;
    ensure_executable(&executable)?;
    required_file(root, "runtime-requirements.txt", &declared)?;
    let package_root = runtime_path(root, package_root)?;
    if !checked_metadata(&package_root)?.is_dir() {
        return Err(RuntimeError::InvalidPublication(
            "package root is not a directory".into(),
        ));
    }
    verify_directory_contents(root, root, &declared)?;
    Ok((executable, package_root))
}

fn verify_declared_files<'a>(
    root: &Path,
    files: &'a [RuntimeFile],
    cancellation: &CancellationToken,
) -> Result<HashMap<&'a str, &'a RuntimeFile>, RuntimeError> {
    if files.len() > MAX_RUNTIME_FILES {
        return Err(RuntimeError::InvalidPublication(
            "manifest declares too many files".into(),
        ));
    }
    let mut total = 0_u64;
    let mut declared = HashMap::with_capacity(files.len());
    for file in files {
        validate_relative_path(&file.path)?;
        total = total
            .checked_add(file.size)
            .filter(|value| *value <= MAX_RUNTIME_TOTAL_BYTES)
            .ok_or_else(|| RuntimeError::InvalidPublication("runtime size exceeds limit".into()))?;
        if file.path == MANIFEST_NAME
            || file.size > MAX_RUNTIME_FILE_BYTES
            || !valid_sha256(&file.sha256)
            || declared.insert(file.path.as_str(), file).is_some()
        {
            return Err(RuntimeError::InvalidPublication(
                "invalid or duplicate file entry".into(),
            ));
        }
        verify_file(root, file, cancellation)?;
    }
    Ok(declared)
}

fn required_file(
    root: &Path,
    relative: &str,
    declared: &HashMap<&str, &RuntimeFile>,
) -> Result<PathBuf, RuntimeError> {
    if !declared.contains_key(relative) {
        return Err(RuntimeError::InvalidPublication(format!(
            "manifest does not declare {relative}"
        )));
    }
    runtime_path(root, relative)
}

fn verify_file(
    root: &Path,
    file: &RuntimeFile,
    cancellation: &CancellationToken,
) -> Result<(), RuntimeError> {
    let path = runtime_path(root, &file.path)?;
    let metadata = checked_metadata(&path)?;
    if !metadata.is_file()
        || metadata.len() != file.size
        || crate::local_resources::verification::sha256_file(&path, cancellation)? != file.sha256
    {
        return Err(RuntimeError::InvalidPublication(format!(
            "file mismatch: {}",
            file.path
        )));
    }
    Ok(())
}

fn verify_directory_contents<'a>(
    root: &Path,
    directory: &Path,
    declared: &HashMap<&'a str, &'a RuntimeFile>,
) -> Result<(), RuntimeError> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        let metadata = checked_metadata(&path)?;
        if metadata.is_dir() {
            verify_directory_contents(root, &path, declared)?;
        } else if !metadata.is_file() {
            return Err(RuntimeError::InvalidPublication(format!(
                "special file: {}",
                path.display()
            )));
        } else {
            let relative = relative_path(root, &path)?;
            if relative != MANIFEST_NAME && !declared.contains_key(relative.as_str()) {
                return Err(RuntimeError::InvalidPublication(format!(
                    "undeclared file: {relative}"
                )));
            }
        }
    }
    Ok(())
}

fn relative_path(root: &Path, path: &Path) -> Result<String, RuntimeError> {
    path.strip_prefix(root)
        .map_err(|error| RuntimeError::Infrastructure(error.to_string()))
        .map(|path| path.to_string_lossy().replace('\\', "/"))
}

fn runtime_path(root: &Path, relative: &str) -> Result<PathBuf, RuntimeError> {
    validate_relative_path(relative)?;
    Ok(relative
        .split('/')
        .fold(root.to_path_buf(), |path, part| path.join(part)))
}

fn validate_relative_path(path: &str) -> Result<(), RuntimeError> {
    if path.is_empty()
        || path.contains('\\')
        || path.starts_with('/')
        || path
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
        || path
            .split('/')
            .next()
            .is_some_and(|part| part.contains(':'))
        || Path::new(path)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(RuntimeError::InvalidPublication(format!(
            "unsafe path: {path}"
        )));
    }
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub(super) fn checked_metadata(path: &Path) -> Result<fs::Metadata, RuntimeError> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            RuntimeError::InvalidPublication(format!(
                "required runtime file is missing: {}",
                path.display()
            ))
        } else {
            error.into()
        }
    })?;
    if metadata.file_type().is_symlink() || is_windows_reparse_point(&metadata) {
        return Err(RuntimeError::InvalidPublication(format!(
            "link is not allowed: {}",
            path.display()
        )));
    }
    Ok(metadata)
}

#[cfg(unix)]
fn ensure_executable(path: &Path) -> Result<(), RuntimeError> {
    use std::os::unix::fs::PermissionsExt;
    if checked_metadata(path)?.permissions().mode() & 0o111 == 0 {
        return Err(RuntimeError::InvalidPublication(
            "interpreter is not executable".into(),
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn ensure_executable(_: &Path) -> Result<(), RuntimeError> {
    Ok(())
}

#[cfg(windows)]
fn is_windows_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & 0x400 != 0
}

#[cfg(not(windows))]
fn is_windows_reparse_point(_: &fs::Metadata) -> bool {
    false
}
