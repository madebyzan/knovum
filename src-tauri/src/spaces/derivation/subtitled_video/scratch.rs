use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

/// Derivation scratch file whose removal is guaranteed on every exit path,
/// including `?` early returns from the render pipeline.
pub(super) struct ScratchFile {
    path: PathBuf,
    retained: bool,
}

impl ScratchFile {
    pub(super) fn new(path: PathBuf) -> Self {
        Self {
            path,
            retained: false,
        }
    }

    pub(super) fn path(&self) -> &Path {
        &self.path
    }

    pub(super) fn retain(mut self) -> PathBuf {
        self.retained = true;
        self.path.clone()
    }
}

impl Drop for ScratchFile {
    fn drop(&mut self) {
        if self.retained {
            return;
        }
        if let Err(error) = fs::remove_file(&self.path) {
            if error.kind() != ErrorKind::NotFound {
                log::warn!(
                    "failed to remove subtitle derivation scratch file {}: {error}",
                    self.path.display()
                );
            }
        }
    }
}
