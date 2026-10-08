use sqlx::{Sqlite, Transaction};

use super::video_config::SpeechMediaSource;
use crate::spaces::db;

pub(crate) async fn is_owned(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    kinds: &[&str],
    source: &SpeechMediaSource,
) -> Result<bool, String> {
    sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spaces_nodes WHERE space_id=? AND kind IN (SELECT value FROM json_each(?)) AND deleted_at IS NULL \
         AND json_extract(data,'$.assetPath')=? AND json_extract(data,'$.sourceRevision')=? \
         AND json_extract(data,'$.mediaAnalysis.sourceRevision')=? \
         AND json_extract(data,'$.mediaAnalysis.durationMs')=?)",
    ).bind(space_id).bind(serde_json::to_string(kinds).map_err(db::json_error)?).bind(&source.asset_path).bind(&source.source_revision)
        .bind(&source.source_revision).bind(i64::try_from(source.duration_ms).map_err(|_| "Media duration is too large")?)
        .fetch_one(&mut **tx).await.map_err(db::sql_error)
}
