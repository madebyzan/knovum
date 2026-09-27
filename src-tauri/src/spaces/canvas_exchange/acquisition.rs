use crate::spaces::assets::{
    self,
    import::{commit::AssetImportProgress, PreparedSpaceImport},
    PreparedAsset,
};
use std::path::{Component, Path, PathBuf};
use tauri::{ipc::Channel, AppHandle};

pub(super) struct ImportResources<'a> {
    pub app: &'a AppHandle,
    pub directory: &'a Path,
    pub progress: &'a Channel<AssetImportProgress>,
    pub acquired: &'a mut Vec<String>,
    pub cancellation: &'a tokio_util::sync::CancellationToken,
}

impl ImportResources<'_> {
    pub fn acquire(&mut self, relative: &str) -> Result<PreparedAsset, String> {
        super::import_control::check(self.cancellation)?;
        let source = resolve_resource(self.directory, relative)?;
        let PreparedSpaceImport { job_id, asset } =
            assets::import::prepare_owned_asset(self.app, source, self.cancellation.child_token())?;
        let Some(job_id) = job_id else {
            return Ok(asset);
        };
        let copied = assets::import::commit::copy_pending_import(&job_id, self.progress).map_err(
            |error| {
                if error == assets::enrichment::ASSET_PROCESSING_CANCELLED {
                    "json-canvas-import-cancelled: JSON Canvas import cancelled".to_owned()
                } else {
                    error
                }
            },
        )?;
        if let Some(path) = &copied.path {
            self.acquired.push(path.clone());
        }
        if let Some(path) = &copied.preview_path {
            self.acquired.push(path.clone());
        }
        Ok(copied)
    }
}

pub(super) fn resolve_resource(directory: &Path, relative: &str) -> Result<PathBuf, String> {
    let relative = Path::new(relative);
    if relative
        .components()
        .any(|part| !matches!(part, Component::Normal(_) | Component::CurDir))
        || relative.as_os_str().is_empty()
    {
        return Err("JSON Canvas resource must stay within its source directory".into());
    }
    let source = directory
        .join(relative)
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if !source.starts_with(directory) || !source.is_file() {
        return Err("JSON Canvas resource escapes its directory or is not a file".into());
    }
    Ok(source)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_absolute_and_traversing_resource_references() {
        for reference in [
            "/tmp/image.png",
            "../image.png",
            "assets/../../image.png",
            "",
        ] {
            assert!(resolve_resource(Path::new("/tmp/canvas"), reference)
                .unwrap_err()
                .contains("source directory"));
        }
    }
}
