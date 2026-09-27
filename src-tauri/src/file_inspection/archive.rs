use std::{fs::File, io::Read, path::Path};

use flate2::read::GzDecoder;
use tar::Archive as TarArchive;
use zip::ZipArchive;

use super::ArchiveMember;

const MAX_ARCHIVE_ENTRIES: usize = 180;
const MAX_TAR_SCAN_BYTES: u64 = 512 * 1024 * 1024;

pub fn archive_entries(path: &Path, logical_name: &str) -> Result<Vec<ArchiveMember>, String> {
    let lower = logical_name.to_ascii_lowercase();
    if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        let file = File::open(path).map_err(|error| error.to_string())?;
        return tar_entries(GzDecoder::new(file).take(MAX_TAR_SCAN_BYTES));
    }
    if lower.ends_with(".tar") {
        let file = File::open(path).map_err(|error| error.to_string())?;
        return tar_entries(file.take(MAX_TAR_SCAN_BYTES));
    }
    zip_entries(path)
}

fn zip_entries(path: &Path) -> Result<Vec<ArchiveMember>, String> {
    let file = File::open(path).map_err(|error| error.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|error| error.to_string())?;
    let count = archive.len().min(MAX_ARCHIVE_ENTRIES);
    let mut entries = Vec::with_capacity(count);
    for index in 0..count {
        let entry = archive
            .by_index_raw(index)
            .map_err(|error| error.to_string())?;
        let decoded = decode_entry_name(entry.name_raw(), entry.name());
        let name = decoded.trim_start_matches('/').replace('\\', "/");
        if name.is_empty() || name.split('/').any(|part| part == "..") {
            continue;
        }
        entries.push(ArchiveMember {
            path: name,
            kind: if entry.is_dir() {
                "folder".to_owned()
            } else {
                "file".to_owned()
            },
            size: (!entry.is_dir()).then_some(entry.size()),
        });
    }
    Ok(entries)
}

fn decode_entry_name(raw: &[u8], fallback: &str) -> String {
    if let Ok(utf8) = std::str::from_utf8(raw) {
        return utf8.to_owned();
    }
    let (decoded, _, had_errors) = encoding_rs::GB18030.decode(raw);
    if !had_errors {
        return decoded.into_owned();
    }
    fallback.to_owned()
}

fn tar_entries(reader: impl Read) -> Result<Vec<ArchiveMember>, String> {
    let mut archive = TarArchive::new(reader);
    let mut entries = Vec::new();
    for entry in archive.entries().map_err(|error| error.to_string())? {
        if entries.len() >= MAX_ARCHIVE_ENTRIES {
            break;
        }
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path().map_err(|error| error.to_string())?;
        let fallback = path.to_string_lossy();
        let decoded = decode_entry_name(&entry.path_bytes(), &fallback);
        let name = decoded.trim_start_matches('/').replace('\\', "/");
        if name.is_empty() || name.split('/').any(|part| part == "..") {
            continue;
        }
        let entry_type = entry.header().entry_type();
        if !(entry_type.is_file() || entry_type.is_dir()) {
            continue;
        }
        entries.push(ArchiveMember {
            path: name,
            kind: if entry_type.is_dir() {
                "folder".to_owned()
            } else {
                "file".to_owned()
            },
            size: entry_type.is_file().then_some(entry.size()),
        });
    }
    Ok(entries)
}
