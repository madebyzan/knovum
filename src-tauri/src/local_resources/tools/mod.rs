use std::path::PathBuf;

use tauri::Manager;

fn bundled_runtime_dir(app: &tauri::AppHandle) -> Option<PathBuf> {
    app.path()
        .resource_dir()
        .ok()
        .map(|directory| directory.join("resources").join("bundled-runtime"))
}

pub fn resolve_tool(
    app: &tauri::AppHandle,
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

pub const fn ffmpeg_name() -> &'static str {
    executable("ffmpeg", "ffmpeg.exe")
}

pub const fn yt_dlp_name() -> &'static str {
    executable("yt-dlp", "yt-dlp.exe")
}

pub const fn deno_name() -> &'static str {
    executable("deno", "deno.exe")
}

pub const fn whisper_cli_name() -> &'static str {
    executable("whisper-cli", "whisper-cli.exe")
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
    } else if cfg!(all(target_os = "windows", target_arch = "aarch64")) {
        "windows-arm64"
    } else if cfg!(all(target_os = "windows", target_arch = "x86_64")) {
        "windows-x64"
    } else if cfg!(all(target_os = "linux", target_arch = "aarch64")) {
        "linux-arm64"
    } else if cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        "linux-x64"
    } else {
        "unsupported"
    }
}
