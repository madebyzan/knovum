pub(crate) mod commands;
pub(crate) mod commit;
pub(super) mod copy;
pub(super) mod geometry;
mod markdown;
pub(crate) mod media;
mod prepare;
pub(crate) mod registry;
pub(crate) mod web;

pub(crate) use prepare::PreparedSpaceImport;

pub(crate) use commit::{copy_owned_import, discard_owned_import};

pub(crate) fn prepare_owned_asset(
    app: &tauri::AppHandle,
    source: std::path::PathBuf,
    cancellation: tokio_util::sync::CancellationToken,
) -> Result<PreparedSpaceImport, String> {
    prepare::prepare_owned_asset(app, source, cancellation)
}
