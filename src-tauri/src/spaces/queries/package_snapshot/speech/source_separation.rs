use serde_json::{json, Value};
use sqlx::{Sqlite, Transaction};

use super::super::{
    decode::{decode_integer, decode_text},
    package_assets::PackageAssets,
};
use crate::spaces::db;

pub(super) async fn load(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    assets: &mut PackageAssets,
) -> Result<Vec<Value>, String> {
    let rows = sqlx::query(
        "SELECT s.* FROM spaces_speech_source_separations s JOIN spaces_nodes n ON n.id=s.source_node_id AND n.space_id=s.space_id AND n.deleted_at IS NULL WHERE s.space_id=? ORDER BY s.created_at,s.id",
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
        "runId":decode_text(row,"run_id")?, "sourceNodeId":decode_text(row,"source_node_id")?, "sourceAssetPath":assets.reference(&decode_text(row,"source_asset_path")?),
        "sourceRevision":decode_text(row,"source_revision")?, "sourceDurationMs":decode_integer(row,"source_duration_ms")?,
        "modelSpecId":decode_text(row,"model_spec_id")?, "backgroundAssetPath":assets.reference(&decode_text(row,"background_asset_path")?),
        "vocalsAssetPath":assets.reference(&decode_text(row,"vocals_asset_path")?), "vocalsSha256":decode_text(row,"vocals_sha256")?,
        "backgroundSha256":decode_text(row,"background_sha256")?, "backgroundSampleRate":decode_integer(row,"background_sample_rate")?,
        "backgroundSampleCount":decode_integer(row,"background_sample_count")?, "backgroundDurationMs":decode_integer(row,"background_duration_ms")?,
        "createdAt":decode_integer(row,"created_at")?,
    }))
}
