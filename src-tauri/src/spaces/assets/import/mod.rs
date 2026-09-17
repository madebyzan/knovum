pub(crate) mod commands;
mod commit;
pub(super) mod copy;
pub(super) mod geometry;
mod markdown;
pub(crate) mod media;
mod prepare;
mod registry;
pub(crate) mod web;

pub(crate) use prepare::PreparedSpaceImport;

pub(crate) fn prepare_owned_asset(
    app: &tauri::AppHandle,
    source: std::path::PathBuf,
) -> Result<PreparedSpaceImport, String> {
    prepare::prepare_owned_asset(app, source)
}
