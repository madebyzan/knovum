use std::{
    fs,
    path::{Path, PathBuf},
};
use tokio_util::sync::CancellationToken;

pub(crate) fn remove_directory(path: &Path) -> Result<(), String> {
    match fs::remove_dir_all(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

pub(crate) async fn verify_file(
    path: PathBuf,
    expected_sha256: String,
    cancellation: CancellationToken,
) -> Result<bool, String> {
    crate::blocking::run(move || {
        super::verification::verify_cancellable(
            &path,
            &expected_sha256,
            &cancellation,
            super::preparation::RESOURCE_PREPARATION_CANCELLED,
        )
    })
    .await
}
