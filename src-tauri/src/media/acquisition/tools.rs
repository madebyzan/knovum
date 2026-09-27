use std::path::PathBuf;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use crate::local_resources::preparation::contracts::ResourcePreparationProgress;
use crate::local_resources::tools;

pub(crate) struct AcquisitionTools {
    pub(crate) yt_dlp: PathBuf,
    pub(crate) deno: PathBuf,
    pub(crate) ffmpeg: PathBuf,
}

pub(crate) async fn prepare_required_for_job(
    app: &AppHandle,
    job_id: &str,
    title: &str,
    business_cancellation: &CancellationToken,
    progress: &mut impl FnMut(ResourcePreparationProgress),
) -> Result<AcquisitionTools, String> {
    let ffmpeg = resolve_bundled(app, "KNOVUM_FFMPEG_PATH", tools::ffmpeg_name(), "FFmpeg")?;
    let prepared = tools::prepare_media_import_tools_for_job(
        app,
        job_id,
        title,
        business_cancellation,
        progress,
    )
    .await?;
    Ok(AcquisitionTools {
        yt_dlp: prepared.yt_dlp,
        deno: prepared.deno,
        ffmpeg,
    })
}

fn resolve_bundled(
    app: &AppHandle,
    env_key: &str,
    file_name: &str,
    label: &str,
) -> Result<PathBuf, String> {
    tools::resolve_tool(app, env_key, file_name)?
        .ok_or_else(|| format!("The bundled {label} tool is unavailable"))
}
