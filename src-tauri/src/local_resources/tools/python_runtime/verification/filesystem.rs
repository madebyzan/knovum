use std::{
    collections::HashMap,
    fs,
    io::Read,
    path::{Component, Path, PathBuf},
};

use sha2::{Digest, Sha256};

use super::{RuntimeFile, MANIFEST_NAME};

const MAX_RUNTIME_FILE_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const MAX_RUNTIME_TOTAL_BYTES: u64 = 16 * 1024 * 1024 * 1024;
const MAX_RUNTIME_FILES: usize = 100_000;

pub(super) fn verify(
    root: &Path,
    files: &[RuntimeFile],
    executable: &str,
    package_root: &str,
) -> Result<(PathBuf, PathBuf), String> {
    let declared = verify_declared_files(root, files)?;
    let executable = required_file(root, executable, &declared)?;
    ensure_executable(&executable)?;
    required_file(root, "packages/knovum_inference/__main__.py", &declared)?;
    let package_root = runtime_path(root, package_root)?;
    if !checked_metadata(&package_root)?.is_dir() {
        return Err("python-runtime-invalid: package root is not a directory".to_string());
    }
    verify_directory_contents(root, root, &declared)?;
    Ok((executable, package_root))
}

fn verify_declared_files<'a>(
    root: &Path,
    files: &'a [RuntimeFile],
) -> Result<HashMap<&'a str, &'a RuntimeFile>, String> {
    if files.len() > MAX_RUNTIME_FILES {
        return Err("python-runtime-invalid: manifest declares too many files".to_string());
    }
    let mut total = 0_u64;
    let mut declared = HashMap::with_capacity(files.len());
    for file in files {
        validate_relative_path(&file.path)?;
        total = total
            .checked_add(file.size)
            .filter(|value| *value <= MAX_RUNTIME_TOTAL_BYTES)
            .ok_or_else(|| "python-runtime-invalid: runtime size exceeds limit".to_string())?;
        if file.path == MANIFEST_NAME
            || file.size > MAX_RUNTIME_FILE_BYTES
            || !valid_sha256(&file.sha256)
            || declared.insert(file.path.as_str(), file).is_some()
        {
            return Err("python-runtime-invalid: invalid or duplicate file entry".to_string());
        }
        verify_file(root, file)?;
    }
    Ok(declared)
}

fn required_file(
    root: &Path,
    relative: &str,
    declared: &HashMap<&str, &RuntimeFile>,
) -> Result<PathBuf, String> {
    if !declared.contains_key(relative) {
        return Err(format!(
            "python-runtime-invalid: manifest does not declare {relative}"
        ));
    }
    runtime_path(root, relative)
}

fn verify_file(root: &Path, file: &RuntimeFile) -> Result<(), String> {
    let path = runtime_path(root, &file.path)?;
    let metadata = checked_metadata(&path)?;
    if !metadata.is_file() || metadata.len() != file.size || sha256_file(&path)? != file.sha256 {
        return Err(format!(
            "python-runtime-invalid: file mismatch: {}",
            file.path
        ));
    }
    Ok(())
}

fn verify_directory_contents<'a>(
    root: &Path,
    directory: &Path,
    declared: &HashMap<&'a str, &'a RuntimeFile>,
) -> Result<(), String> {
    for entry in fs::read_dir(directory).map_err(|error| error.to_string())? {
        let path = entry.map_err(|error| error.to_string())?.path();
        let metadata = checked_metadata(&path)?;
        if metadata.is_dir() {
            verify_directory_contents(root, &path, declared)?;
        } else if !metadata.is_file() {
            return Err(format!(
                "python-runtime-invalid: special file: {}",
                path.display()
            ));
        } else {
            let relative = relative_path(root, &path)?;
            if relative != MANIFEST_NAME && !declared.contains_key(relative.as_str()) {
                return Err(format!(
                    "python-runtime-invalid: undeclared file: {relative}"
                ));
            }
        }
    }
    Ok(())
}

fn relative_path(root: &Path, path: &Path) -> Result<String, String> {
    path.strip_prefix(root)
        .map_err(|error| error.to_string())
        .map(|path| path.to_string_lossy().replace('\\', "/"))
}

fn runtime_path(root: &Path, relative: &str) -> Result<PathBuf, String> {
    validate_relative_path(relative)?;
    Ok(relative
        .split('/')
        .fold(root.to_path_buf(), |path, part| path.join(part)))
}

fn validate_relative_path(path: &str) -> Result<(), String> {
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
        return Err(format!("python-runtime-invalid: unsafe path: {path}"));
    }
    Ok(())
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = [0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            return Ok(format!("{:x}", hash.finalize()));
        }
        hash.update(&buffer[..read]);
    }
}

pub(super) fn checked_metadata(path: &Path) -> Result<fs::Metadata, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if metadata.file_type().is_symlink() || is_windows_reparse_point(&metadata) {
        return Err(format!(
            "python-runtime-invalid: link is not allowed: {}",
            path.display()
        ));
    }
    Ok(metadata)
}

#[cfg(unix)]
fn ensure_executable(path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    if checked_metadata(path)?.permissions().mode() & 0o111 == 0 {
        return Err("python-runtime-invalid: interpreter is not executable".to_string());
    }
    Ok(())
}

#[cfg(not(unix))]
fn ensure_executable(_: &Path) -> Result<(), String> {
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
