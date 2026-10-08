use std::path::PathBuf;

#[cfg(not(debug_assertions))]
use tauri::Manager;
use tauri::{AppHandle, Runtime};

mod catalog;
mod coordinator;
mod deno_archive;
mod filesystem;
mod installer;
#[cfg(test)]
mod tests;

pub(crate) use coordinator::prepare_media_import_tools_for_job;
pub(crate) use coordinator::prepare_media_import_tools_for_request;

#[derive(Clone)]
pub(crate) struct ToolPreparationProgress {
    pub(crate) stage: super::preparation::contracts::ResourcePreparationStage,
    pub(crate) downloaded: u64,
    pub(crate) total: Option<u64>,
}

pub(crate) struct PreparedMediaImportTools {
    pub(crate) yt_dlp: PathBuf,
    pub(crate) deno: PathBuf,
}

#[cfg(debug_assertions)]
fn bundled_runtime_dir<R: Runtime>(_: &AppHandle<R>) -> Result<PathBuf, String> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/bundled-runtime"))
}

#[cfg(not(debug_assertions))]
fn bundled_runtime_dir<R: Runtime>(app: &AppHandle<R>) -> Result<PathBuf, String> {
    app.path()
        .resource_dir()
        .map(|directory| directory.join("resources/bundled-runtime"))
        .map_err(|error| error.to_string())
}

pub fn resolve_ffmpeg<R: Runtime>(app: &AppHandle<R>) -> Result<Option<PathBuf>, String> {
    let directory = bundled_runtime_dir(app)?;
    let candidate = directory.join(platform_directory()).join(ffmpeg_name());
    Ok(candidate.is_file().then_some(candidate))
}

pub(crate) fn resolve_media_import_tool<R: Runtime>(
    app: &AppHandle<R>,
    kind: catalog::MediaImportToolKind,
) -> Result<Option<PathBuf>, String> {
    let Some(descriptor) = catalog::find_descriptor(kind)? else {
        return Ok(None);
    };
    filesystem::verified_installation_path(app, &descriptor)
}

pub(crate) fn resolve_yt_dlp<R: Runtime>(app: &AppHandle<R>) -> Result<Option<PathBuf>, String> {
    resolve_media_import_tool(app, catalog::MediaImportToolKind::YtDlp)
}

pub(crate) fn resolve_deno<R: Runtime>(app: &AppHandle<R>) -> Result<Option<PathBuf>, String> {
    resolve_media_import_tool(app, catalog::MediaImportToolKind::Deno)
}

pub const fn ffmpeg_name() -> &'static str {
    executable("bin/ffmpeg", "Library/bin/ffmpeg.exe")
}

pub const fn yt_dlp_name() -> &'static str {
    executable("yt-dlp", "yt-dlp.exe")
}

pub const fn deno_name() -> &'static str {
    executable("deno", "deno.exe")
}

const fn executable(unix: &'static str, windows: &'static str) -> &'static str {
    if cfg!(target_os = "windows") {
        windows
    } else {
        unix
    }
}

pub const fn platform_directory() -> &'static str {
    if cfg!(all(target_os = "macos", target_arch = "aarch64")) {
        "macos-arm64"
    } else if cfg!(all(target_os = "macos", target_arch = "x86_64")) {
        "macos-x64"
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "windows-x64"
    } else {
        "unsupported"
    }
}
