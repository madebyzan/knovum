use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
};

use sha2::{Digest, Sha256};
use tokio_util::sync::CancellationToken;

use crate::file_inspection::verify_content_revision;

use super::{
    audio_preparation::{inspect_prepared_audio, prepare_audio, PreparedAudio},
    workspace::AnalysisWorkspace,
    TRANSCRIPTION_CANCELLED,
};

// Preparation is shared by independent analysis/transcription jobs. Serializing only the
// decode phase prevents two jobs from publishing the same cache entry concurrently.
static PREPARATION: Mutex<()> = Mutex::new(());

pub(crate) struct PreparedMediaAudio {
    pub wav: PathBuf,
    pub analysis: PreparedAudio,
}

/// A disposable, source-revision-addressed cache. It contains no Space or model state.
/// The caller owns the cache root and clears it during startup before work is resumed.
pub fn prepare(
    ffmpeg: &Path,
    source: &Path,
    source_revision: &str,
    root: &Path,
    cancellation: &CancellationToken,
) -> Result<PreparedMediaAudio, String> {
    let _guard = PREPARATION
        .lock()
        .map_err(|_| "Audio preparation state is unavailable")?;
    if cancellation.is_cancelled() {
        return Err(TRANSCRIPTION_CANCELLED.into());
    }
    verify_content_revision(source, source_revision)?;
    let key = format!(
        "decoded-v1-{:x}",
        Sha256::digest(source_revision.as_bytes())
    );
    let target = root.join(key);
    if target.exists() {
        let paths = AnalysisWorkspace::at(target);
        let analysis = inspect_prepared_audio(&paths, cancellation)?;
        return Ok(PreparedMediaAudio {
            wav: paths.wav,
            analysis,
        });
    }
    let paths = AnalysisWorkspace::create(root)?;
    let result = prepare_audio(ffmpeg, source, &paths, cancellation).and_then(|analysis| {
        if cancellation.is_cancelled() {
            return Err(TRANSCRIPTION_CANCELLED.into());
        }
        // Never publish a decode under the old key if its source changed while FFmpeg ran.
        verify_content_revision(source, source_revision)?;
        fs::rename(paths.directory(), &target)
            .map_err(|error| format!("failed to publish prepared audio: {error}"))?;
        Ok(PreparedMediaAudio {
            wav: AnalysisWorkspace::at(target).wav,
            analysis,
        })
    });
    if let Err(error) = paths.cleanup() {
        return Err(match result {
            Ok(_) => error,
            Err(cause) => format!("{cause}; audio preparation cleanup failed: {error}"),
        });
    }
    result
}
