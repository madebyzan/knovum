use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

pub struct ScratchFile {
    path: PathBuf,
    retained: bool,
}

impl ScratchFile {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            retained: false,
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn retain(mut self) -> PathBuf {
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
                    "failed to remove scratch file {}: {error}",
                    self.path.display()
                );
            }
        }
    }
}
