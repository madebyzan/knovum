use std::path::PathBuf;

use tauri::{AppHandle, Manager, Runtime};

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

fn bundled_runtime_dir<R: Runtime>(app: &AppHandle<R>) -> Option<PathBuf> {
    app.path()
        .resource_dir()
        .ok()
        .map(|directory| directory.join("resources").join("bundled-runtime"))
}

pub fn resolve_tool<R: Runtime>(
    app: &AppHandle<R>,
    env_key: &str,
    file_name: &str,
) -> Result<Option<PathBuf>, String> {
    if let Some(path) = super::configured_path::configured_path(env_key)? {
        return Ok(Some(path));
    }
    let Some(directory) = bundled_runtime_dir(app) else {
        return Ok(None);
    };
    let candidate = directory.join(platform_directory()).join(file_name);
    Ok(candidate.is_file().then_some(candidate))
}

pub(crate) fn resolve_media_import_tool<R: Runtime>(
    app: &AppHandle<R>,
    kind: catalog::MediaImportToolKind,
) -> Result<Option<PathBuf>, String> {
    let Some(descriptor) = catalog::find_descriptor(kind)? else {
        return Ok(None);
    };
    if let Some(path) = super::configured_path::configured_path(descriptor.env_key)? {
        return Ok(Some(path));
    }
    filesystem::verified_installation_path(app, &descriptor)
}

pub(crate) fn resolve_yt_dlp<R: Runtime>(app: &AppHandle<R>) -> Result<Option<PathBuf>, String> {
    resolve_media_import_tool(app, catalog::MediaImportToolKind::YtDlp)
}

pub(crate) fn resolve_deno<R: Runtime>(app: &AppHandle<R>) -> Result<Option<PathBuf>, String> {
    resolve_media_import_tool(app, catalog::MediaImportToolKind::Deno)
}

pub const fn ffmpeg_name() -> &'static str {
    executable("ffmpeg", "ffmpeg.exe")
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
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "windows-x64"
    } else {
        "unsupported"
    }
}
