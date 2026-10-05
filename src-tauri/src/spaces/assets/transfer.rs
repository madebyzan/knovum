use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use tokio_util::sync::CancellationToken;

use super::import::copy;

pub(super) fn copy_then<T>(
    source: &Path,
    target: &Path,
    finish: impl FnOnce() -> Result<T, String>,
) -> Result<T, String> {
    let transfer = copy::copy_resource(source, target, &CancellationToken::new(), |_| Ok(()))
        .and_then(|_| finish());
    match transfer {
        Ok(value) => Ok(value),
        Err(error) => match remove_path(target) {
            Ok(()) => Err(error),
            Err(cleanup_error) => Err(format!("{error}; cleanup also failed: {cleanup_error}")),
        },
    }
}

pub(super) fn imported_resource_name(source: &Path) -> Result<String, String> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_nanos();
    Ok(match source.extension().and_then(|value| value.to_str()) {
        Some(extension) => format!("{stamp}.{}", extension.to_ascii_lowercase()),
        None => stamp.to_string(),
    })
}

pub(super) fn remove_path(path: &Path) -> Result<(), String> {
    let removed = if path.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };
    match removed {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}
