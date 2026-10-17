use std::{fs, path::Path};

use crate::{foundation::scratch_file::ScratchFile, spaces::assets::authorize_destination};

/// The destination is the path a native save dialog returned, so replacing an
/// existing file is what the user already confirmed. A failed write leaves no
/// half-written file where a complete one is expected.
pub(super) fn write(
    app: &tauri::AppHandle,
    destination: &Path,
    bytes: &[u8],
) -> Result<(), String> {
    authorize_destination(app, destination)?;
    let file = ScratchFile::new(destination.to_owned());
    fs::write(destination, bytes).map_err(|error| error.to_string())?;
    file.retain();
    Ok(())
}

pub(super) fn copy(
    app: &tauri::AppHandle,
    source: &Path,
    destination: &Path,
) -> Result<(), String> {
    let canonical = crate::spaces::assets::authorize_spaces_asset(app, source)?;
    authorize_destination(app, destination)?;
    let file = ScratchFile::new(destination.to_owned());
    fs::copy(&canonical, destination).map_err(|error| error.to_string())?;
    file.retain();
    Ok(())
}
