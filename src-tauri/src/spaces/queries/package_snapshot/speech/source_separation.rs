use serde_json::{json, Value};
use sqlx::{Sqlite, Transaction};

use super::super::{decode::*, package_assets::PackageAssets};
use crate::spaces::db;

pub(super) async fn load(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    assets: &mut PackageAssets,
) -> Result<Vec<Value>, String> {
    let rows = sqlx::query(
        "SELECT * FROM spaces_speech_source_separations WHERE space_id=? ORDER BY created_at,id",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    rows.iter().map(|row| map(row, assets)).collect()
}

fn map(row: &sqlx::sqlite::SqliteRow, assets: &mut PackageAssets) -> Result<Value, String> {
    Ok(json!({
        "id":decode_text(row,"id")?, "spaceId":decode_text(row,"space_id")?,
        "runId":decode_text(row,"run_id")?, "sourceAssetPath":assets.reference(&decode_text(row,"source_asset_path")?),
        "sourceRevision":decode_text(row,"source_revision")?, "sourceDurationMs":decode_integer(row,"source_duration_ms")?,
        "modelSpecId":decode_text(row,"model_spec_id")?, "backgroundAssetPath":assets.reference(&decode_text(row,"background_asset_path")?),
        "vocalsAssetPath":assets.reference(&decode_text(row,"vocals_asset_path")?), "vocalsSha256":decode_text(row,"vocals_sha256")?,
        "backgroundSha256":decode_text(row,"background_sha256")?, "backgroundSampleRate":decode_integer(row,"background_sample_rate")?,
        "backgroundSampleCount":decode_integer(row,"background_sample_count")?, "backgroundDurationMs":decode_integer(row,"background_duration_ms")?,
        "reviewStatus":decode_text(row,"review_status")?, "reviewWarning":decode_text(row,"review_warning")?,
        "createdAt":decode_integer(row,"created_at")?, "reviewedAt":decode_optional_integer(row,"reviewed_at")?,
    }))
}
