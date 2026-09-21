use std::{collections::HashMap, path::PathBuf};

use super::super::super::AssetIndex;

pub(super) fn verify_assets(
    targets: &HashMap<String, (PathBuf, bool)>,
    index: &[AssetIndex],
) -> Result<(), String> {
    for asset in index {
        let (path, directory) = targets
            .get(&asset.token)
            .ok_or("Package asset target is missing")?;
        if (*directory && !path.is_dir()) || (!*directory && !path.is_file()) {
            return Err("Space package asset did not materialize".into());
        }
        let expected = asset
            .source_revision
            .as_deref()
            .ok_or("Space package asset digest is missing")?;
        if crate::file_inspection::content_revision(path)? != expected {
            return Err(
                "Space package asset content does not match its SHA-256 sourceRevision".into(),
            );
        }
    }
    Ok(())
}
