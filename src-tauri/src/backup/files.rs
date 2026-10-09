use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::File,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};
use walkdir::WalkDir;

use super::{
    contracts::{ArchiveFile, MAX_FILES, MAX_FILE_BYTES, MAX_TOTAL_BYTES},
    paths,
};

pub fn collect(root: &Path, archive_root: &str) -> Result<Vec<(ArchiveFile, PathBuf)>, String> {
    if !paths::entry_exists(root)? {
        return Ok(Vec::new());
    }
    let root_metadata = std::fs::symlink_metadata(root).map_err(|error| error.to_string())?;
    if !root_metadata.is_dir() || root_metadata.file_type().is_symlink() {
        return Err(format!(
            "Managed asset root must be a real directory: {}",
            root.display()
        ));
    }
    let mut files = Vec::new();
    let mut identities = BTreeSet::new();
    let mut total = 0u64;
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(|error| error.to_string())?;
        let metadata =
            std::fs::symlink_metadata(entry.path()).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() || (!metadata.is_dir() && !metadata.is_file()) {
            return Err(format!(
                "Unsupported managed asset: {}",
                entry.path().display()
            ));
        }
        if !metadata.is_file() {
            continue;
        }
        let relative = entry
            .path()
            .strip_prefix(root)
            .map_err(|error| error.to_string())?;
        let relative = normalized(relative)?;
        let path = format!("{archive_root}/{relative}");
        validate_archive_path(&path)?;
        if !identities.insert(path.to_lowercase()) {
            return Err(format!("Asset path collision: {path}"));
        }
        let size = metadata.len();
        total = total.checked_add(size).ok_or("Backup size overflow")?;
        if size > MAX_FILE_BYTES || total > MAX_TOTAL_BYTES || files.len() >= MAX_FILES {
            return Err("Managed data exceeds backup safety limits".into());
        }
        files.push((
            ArchiveFile {
                path,
                size,
                sha256: hash(entry.path())?,
            },
            entry.path().into(),
        ));
    }
    Ok(files)
}

pub fn hash(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(hex::encode(digest.finalize()))
}

pub fn copy_bounded(
    reader: &mut impl Read,
    destination: &Path,
    size: u64,
) -> Result<String, String> {
    if size > MAX_FILE_BYTES {
        return Err("Archive entry exceeds the file limit".into());
    }
    if let Some(parent) = destination.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|error| error.to_string())?;
    let digest = copy_hashed(reader, &mut output, size)?;
    output.sync_all().map_err(|error| error.to_string())?;
    Ok(digest)
}

pub fn copy_hashed(
    reader: &mut impl Read,
    output: &mut impl Write,
    size: u64,
) -> Result<String, String> {
    if size > MAX_FILE_BYTES {
        return Err("Archive entry exceeds the file limit".into());
    }
    let mut digest = Sha256::new();
    let mut limited = reader.take(size + 1);
    let copied = std::io::copy(&mut limited, &mut DigestWriter(output, &mut digest))
        .map_err(|error| error.to_string())?;
    if copied != size {
        return Err("Archive entry size does not match its manifest".into());
    }
    Ok(hex::encode(digest.finalize()))
}

pub fn validate_archive_path(path: &str) -> Result<(), String> {
    if path.is_empty()
        || path.contains('\\')
        || path.contains(':')
        || path.contains('\0')
        || path.contains("//")
        || Path::new(path).is_absolute()
        || Path::new(path)
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(format!("Unsafe archive path: {path}"));
    }
    for component in path.split('/') {
        if component.ends_with('.') || component.ends_with(' ') || windows_reserved(component) {
            return Err(format!("Archive path is not portable: {path}"));
        }
    }
    Ok(())
}

fn windows_reserved(component: &str) -> bool {
    let stem = component
        .split('.')
        .next()
        .unwrap_or_default()
        .to_ascii_uppercase();
    matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || (stem.len() == 4
            && matches!(&stem[..3], "COM" | "LPT")
            && matches!(stem.as_bytes()[3], b'1'..=b'9'))
}

fn normalized(path: &Path) -> Result<String, String> {
    let parts: Result<Vec<_>, _> = path
        .components()
        .map(|part| match part {
            Component::Normal(value) => value
                .to_str()
                .map(str::to_owned)
                .ok_or("Asset path is not UTF-8"),
            _ => Err("Managed asset path is invalid"),
        })
        .collect();
    Ok(parts?.join("/"))
}

struct DigestWriter<'a, W: Write>(&'a mut W, &'a mut Sha256);
impl<W: Write> Write for DigestWriter<'_, W> {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        let written = self.0.write(buffer)?;
        self.1.update(&buffer[..written]);
        Ok(written)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}
