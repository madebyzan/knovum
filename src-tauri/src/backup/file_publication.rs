use std::path::Path;

pub fn replace_file(source: &Path, destination: &Path) -> Result<(), String> {
    rename(source, destination, true)
}

pub fn rename_new(source: &Path, destination: &Path) -> Result<(), String> {
    match std::fs::symlink_metadata(destination) {
        Ok(_) => {
            return Err(format!(
                "Publication destination already exists: {}",
                destination.display()
            ))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.to_string()),
    }
    rename(source, destination, false)
}

#[cfg(windows)]
fn rename(source: &Path, destination: &Path, replace: bool) -> Result<(), String> {
    use std::{iter, os::windows::ffi::OsStrExt};
    use windows::{
        core::PCWSTR,
        Win32::Storage::FileSystem::{
            MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
        },
    };

    let source: Vec<u16> = source
        .as_os_str()
        .encode_wide()
        .chain(iter::once(0))
        .collect();
    let destination: Vec<u16> = destination
        .as_os_str()
        .encode_wide()
        .chain(iter::once(0))
        .collect();
    let flags = if replace {
        MOVEFILE_WRITE_THROUGH | MOVEFILE_REPLACE_EXISTING
    } else {
        MOVEFILE_WRITE_THROUGH
    };
    unsafe { MoveFileExW(PCWSTR(source.as_ptr()), PCWSTR(destination.as_ptr()), flags) }
        .map_err(|error| error.to_string())
}

#[cfg(not(windows))]
fn rename(source: &Path, destination: &Path, _replace: bool) -> Result<(), String> {
    std::fs::rename(source, destination).map_err(|error| error.to_string())?;
    sync_parent(source)?;
    sync_parent(destination)
}

pub fn remove(path: &Path) -> Result<(), String> {
    if path.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    }
    .map_err(|error| error.to_string())?;
    sync_parent(path)
}

#[cfg(not(windows))]
pub fn sync_tree(root: &Path) -> Result<(), String> {
    for entry in walkdir::WalkDir::new(root).contents_first(true) {
        let entry = entry.map_err(|error| error.to_string())?;
        if entry.file_type().is_dir() {
            std::fs::File::open(entry.path())
                .and_then(|file| file.sync_all())
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

#[cfg(windows)]
pub fn sync_tree(_root: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(not(windows))]
pub fn sync_directory(path: &Path) -> Result<(), String> {
    std::fs::File::open(path)
        .and_then(|file| file.sync_all())
        .map_err(|error| error.to_string())
}

#[cfg(windows)]
pub fn sync_directory(_path: &Path) -> Result<(), String> {
    Ok(())
}

fn sync_parent(path: &Path) -> Result<(), String> {
    sync_directory(
        path.parent()
            .ok_or("Published path has no parent directory")?,
    )
}
