use std::path::Path;

use tokio_util::sync::CancellationToken;

use crate::{
    file_inspection::verify_content_revision,
    local_resources::tools,
    media::{prepared_audio, prepared_audio::PreparedMediaAudio, workspace::analysis_directory},
};

/// The Space owns asset authorization and runtime resolution; decoding stays in the media engine.
pub(crate) fn prepare(
    app: &tauri::AppHandle,
    source: &Path,
    source_revision: &str,
    cancellation: &CancellationToken,
) -> Result<PreparedMediaAudio, String> {
    let source = super::authorize_spaces_asset(app, source)?;
    let ffmpeg = tools::resolve_tool(app, "KNOVUM_FFMPEG_PATH", tools::ffmpeg_name())
        .ok_or("FFmpeg is required for media analysis and is not installed")?;
    let audio = prepared_audio::prepare(
        &ffmpeg,
        &source,
        source_revision,
        &analysis_directory(app)?,
        cancellation,
    )?;
    verify_content_revision(&source, source_revision)?;
    Ok(audio)
}
