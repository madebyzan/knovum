use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{collections::BTreeSet, fs, path::Path};
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Manifest {
    schema_version: u8,
    target: String,
    files: Vec<Entry>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    path: String,
    size: Option<u64>,
    sha256: Option<String>,
    link: Option<String>,
}

pub(super) fn verify(root: &Path, trusted: &[u8]) -> Result<(), String> {
    let manifest_path = root.join("agent-manifest.json");
    let metadata = fs::symlink_metadata(&manifest_path).map_err(|e| e.to_string())?;
    if !metadata.is_file()
        || metadata.len() > 4 * 1024 * 1024
        || fs::read(&manifest_path).map_err(|e| e.to_string())? != trusted
    {
        return Err("agent-runtime-manifest-differs".into());
    }
    let manifest: Manifest = serde_json::from_slice(trusted).map_err(|e| e.to_string())?;
    if manifest.schema_version != 1
        || manifest.target != crate::local_resources::tools::platform_directory()
        || manifest.files.is_empty()
        || manifest.files.len() > 16384
    {
        return Err("agent-runtime-inventory-invalid".into());
    }
    let boundary = root.canonicalize().map_err(|e| e.to_string())?;
    let mut expected = BTreeSet::from(["agent-manifest.json".to_owned()]);
    for file in &manifest.files {
        if !expected.insert(file.path.clone()) {
            return Err("agent-runtime-inventory-duplicate".into());
        }
        let path = super::paths::join(root, &file.path)?;
        if !path
            .canonicalize()
            .map_err(|e| e.to_string())?
            .starts_with(&boundary)
        {
            return Err("agent-runtime-resource-escapes".into());
        }
        let metadata = fs::symlink_metadata(&path).map_err(|e| e.to_string())?;
        if let Some(link) = &file.link {
            if !metadata.is_symlink()
                || file.sha256.is_some()
                || file.size.is_some()
                || fs::read_link(&path).map_err(|e| e.to_string())?.to_str() != Some(link.as_str())
            {
                return Err("agent-runtime-link-differs".into());
            }
        } else {
            if !metadata.is_file() || Some(metadata.len()) != file.size {
                return Err("agent-runtime-resource-differs".into());
            }
            let bytes = fs::read(&path).map_err(|e| e.to_string())?;
            if file.sha256.as_deref() != Some(hex::encode(Sha256::digest(bytes)).as_str()) {
                return Err("agent-runtime-resource-differs".into());
            }
        }
    }
    let mut actual = BTreeSet::new();
    for entry in walkdir::WalkDir::new(root).min_depth(1).follow_links(false) {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().is_dir() {
            continue;
        }
        actual.insert(
            entry
                .path()
                .strip_prefix(root)
                .expect("WalkDir::new(root) guarantees entry paths are descendants of root")
                .to_str()
                .ok_or("agent-runtime-resource-path-invalid")?
                .replace('\\', "/"),
        );
        if actual.len() > 16385 {
            return Err("agent-runtime-inventory-exceeds-bound".into());
        }
    }
    if actual != expected {
        return Err("agent-runtime-inventory-differs".into());
    }
    Ok(())
}
