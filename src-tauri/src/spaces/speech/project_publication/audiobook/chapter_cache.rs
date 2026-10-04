use std::{fs, path::Path};

use tauri::{AppHandle, Runtime};

use super::super::encoding::wav_sample_count;
use super::{cleanup, ChapterCheckpoint};
use crate::spaces::speech::publication::hash_file;
use crate::spaces::{assets, db};

pub(super) async fn load<R: Runtime>(
    app: &AppHandle<R>,
    pool: &sqlx::SqlitePool,
    directory: &Path,
    revision_id: &str,
    chapter_key: &str,
    digest: &str,
    sample_rate: u32,
) -> Result<Option<ChapterCheckpoint>, String> {
    let Some(row) = sqlx::query_as::<_, (String, String, String, i64, i64)>(
        "SELECT id,asset_path,sha256,sample_rate,sample_count \
         FROM spaces_speech_chapter_checkpoints \
         WHERE project_revision_id=? AND chapter_key=? AND input_digest=?",
    )
    .bind(revision_id)
    .bind(chapter_key)
    .bind(digest)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    else {
        return Ok(None);
    };
    let (checkpoint_id, stored_path, sha256, rate, samples) = row;
    cleanup::checkpoint_workspace(directory, digest, &stored_path)?;
    if let Ok(path) = assets::authorize_spaces_asset(app, Path::new(&stored_path)) {
        if hash_file(&path).ok().as_deref() == Some(sha256.as_str())
            && rate == i64::from(sample_rate)
            && wav_sample_count(&path, sample_rate).ok() == u64::try_from(samples).ok()
        {
            return Ok(Some(ChapterCheckpoint {
                chapter_key: chapter_key.into(),
                path: path.to_string_lossy().into_owned(),
                sha256,
                sample_rate,
                sample_count: u64::try_from(samples)
                    .map_err(|_| "Chapter sample count is invalid")?,
            }));
        }
        fs::remove_file(path).map_err(|error| error.to_string())?;
    }
    sqlx::query("DELETE FROM spaces_speech_chapter_checkpoints WHERE id=?")
        .bind(checkpoint_id)
        .execute(pool)
        .await
        .map_err(db::sql_error)?;
    Ok(None)
}
