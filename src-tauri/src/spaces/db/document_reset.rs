use crate::spaces::{
    db,
    knowledge::{chunk_store, indexing::revision},
};
use serde_json::Value;
use sqlx::{Row, SqlitePool};

/// Finish explicit development-report resets before any node reader or asset GC runs.
/// This does not deserialize, upgrade or retain the obsolete manifest contract.
pub(super) async fn complete(pool: &SqlitePool) -> Result<(), String> {
    let mut tx = db::begin_write(pool).await?;
    let rows = sqlx::query("SELECT id,space_id,data FROM spaces_nodes WHERE json_type(data,'$.documentCoverage') IS NOT NULL AND json_extract(data,'$.documentCoverage.schemaVersion') IS NOT 1")
        .fetch_all(&mut *tx).await.map_err(db::sql_error)?;
    for row in rows {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        let space: String = row.try_get("space_id").map_err(db::sql_error)?;
        let raw: String = row.try_get("data").map_err(db::sql_error)?;
        let mut data: Value = serde_json::from_str(&raw).map_err(db::json_error)?;
        data.as_object_mut()
            .ok_or("Document node data must be an object")?
            .remove("documentCoverage");
        revision::apply(&mut data)?;
        chunk_store::detach_canonical(&mut tx, &space, &id).await?;
        chunk_store::clear(&mut tx, &space, &id, "derived").await?;
        sqlx::query("UPDATE spaces_nodes SET data=$1 WHERE space_id=$2 AND id=$3")
            .bind(serde_json::to_string(&data).map_err(db::json_error)?)
            .bind(&space)
            .bind(&id)
            .execute(&mut *tx)
            .await
            .map_err(db::sql_error)?;
    }
    tx.commit().await.map_err(db::sql_error)
}
