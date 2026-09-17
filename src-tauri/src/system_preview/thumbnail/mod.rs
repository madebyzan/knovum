use std::path::Path;

use tokio_util::sync::CancellationToken;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(any(test, target_os = "macos", target_os = "windows"))]
pub(crate) mod policy;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
pub(crate) fn save(
    source: &Path,
    target: &Path,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    macos::save(source, target, cancellation)
}

#[cfg(target_os = "windows")]
pub(crate) fn save(
    source: &Path,
    target: &Path,
    cancellation: &CancellationToken,
) -> Result<(), String> {
    windows::save(source, target, cancellation)
}

fn ensure_not_cancelled(cancellation: &CancellationToken) -> Result<(), String> {
    if cancellation.is_cancelled() {
        Err("SYSTEM_PREVIEW_CANCELLED".to_owned())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
