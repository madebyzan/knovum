use std::{fs, path::Path};

pub(super) fn attempt_artifact_path(
    work_directory: &Path,
    request_id: &str,
    file_name: &str,
) -> Result<std::path::PathBuf, String> {
    let relative = Path::new(file_name);
    let attempt_name = format!("attempt-{request_id}");
    let mut components = relative.components();
    if !matches!(components.next(), Some(std::path::Component::Normal(name)) if name.to_str() == Some(attempt_name.as_str()))
        || !matches!(components.next(), Some(std::path::Component::Normal(_)))
        || components.next().is_some()
        || !file_name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b'/'))
    {
        return Err("invalid_output: worker artifact name is invalid".into());
    }
    let attempt_directory = work_directory.join(&attempt_name);
    checked_directory(&attempt_directory)?;
    Ok(work_directory.join(relative))
}

pub(super) fn checked_directory(path: &Path) -> Result<(), String> {
    let metadata =
        fs::symlink_metadata(path).map_err(|error| format!("invalid_output: {error}"))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() || is_reparse_point(&metadata) {
        return Err("invalid_output: worker attempt directory is invalid".into());
    }
    Ok(())
}

pub(super) fn open_regular(path: &Path) -> Result<fs::File, String> {
    let before = fs::symlink_metadata(path).map_err(|error| format!("invalid_output: {error}"))?;
    if !before.file_type().is_file() || is_reparse_point(&before) {
        return Err("invalid_output: worker artifact is not a regular file".into());
    }
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(target_os = "macos")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(0x0000_0100);
    }
    #[cfg(target_os = "linux")]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(0x0002_0000);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x0020_0000);
    }
    let file = options
        .open(path)
        .map_err(|error| format!("invalid_output: {error}"))?;
    let opened = file
        .metadata()
        .map_err(|error| format!("invalid_output: {error}"))?;
    if !same_file(&before, &opened) || !opened.is_file() {
        return Err("invalid_output: worker artifact changed while opening".into());
    }
    Ok(file)
}

#[cfg(unix)]
fn same_file(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(windows)]
fn same_file(left: &fs::Metadata, right: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    !is_reparse_point(left)
        && !is_reparse_point(right)
        && left.file_size() == right.file_size()
        && left.last_write_time() == right.last_write_time()
}

#[cfg(windows)]
pub(super) fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    metadata.file_attributes() & 0x0000_0400 != 0
}

#[cfg(not(windows))]
pub(super) fn is_reparse_point(_: &fs::Metadata) -> bool {
    false
}

#[cfg(test)]
#[path = "tests/artifact.rs"]
mod tests;
