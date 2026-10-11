use std::path::Path;

use tokio_util::sync::CancellationToken;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(any(test, target_os = "macos", target_os = "windows"))]
pub(crate) mod policy;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "macos")]
pub(crate) fn save_owned(
    source: &Path,
    target: &Path,
    cancellation: &CancellationToken,
    claim_path: &mut dyn FnMut(&Path) -> Result<(), String>,
) -> Result<(), String> {
    macos::save(source, target, cancellation, claim_path)
}

#[cfg(target_os = "windows")]
pub(crate) fn save_owned(
    source: &Path,
    target: &Path,
    cancellation: &CancellationToken,
    claim_path: &mut dyn FnMut(&Path) -> Result<(), String>,
) -> Result<(), String> {
    windows::save(source, target, cancellation, claim_path)
}

fn claim_output_paths(
    target: &Path,
    claim_path: &mut dyn FnMut(&Path) -> Result<(), String>,
) -> Result<std::path::PathBuf, String> {
    claim_path(target)?;
    let partial = target.with_extension("partial");
    claim_path(&partial)?;
    Ok(partial)
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
