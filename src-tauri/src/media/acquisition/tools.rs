use std::path::PathBuf;
use tauri::AppHandle;

use crate::local_resources::tools;

pub(crate) struct AcquisitionTools {
    pub(crate) yt_dlp: PathBuf,
    pub(crate) deno: PathBuf,
    pub(crate) ffmpeg: PathBuf,
}

pub(crate) fn resolve_required(app: &AppHandle) -> Result<AcquisitionTools, String> {
    Ok(AcquisitionTools {
        yt_dlp: resolve(app, "KNOVUM_YT_DLP_PATH", tools::yt_dlp_name(), "yt-dlp")?,
        deno: resolve(app, "KNOVUM_DENO_PATH", tools::deno_name(), "Deno")?,
        ffmpeg: resolve(app, "KNOVUM_FFMPEG_PATH", tools::ffmpeg_name(), "FFmpeg")?,
    })
}

fn resolve(
    app: &AppHandle,
    env_key: &str,
    file_name: &str,
    label: &str,
) -> Result<PathBuf, String> {
    tools::resolve_tool(app, env_key, file_name)?
        .ok_or_else(|| format!("The bundled {label} tool is unavailable"))
}
