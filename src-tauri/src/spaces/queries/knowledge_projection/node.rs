use serde_json::{json, Value};
use sqlx::{Row, SqliteConnection};

use crate::spaces::db;

pub(crate) async fn get_node(
    connection: &mut SqliteConnection,
    space_id: &str,
    node_id: &str,
) -> Result<Value, String> {
    let row = sqlx::query(
        "SELECT id,kind,data,created_at,updated_at FROM spaces_nodes \
         WHERE space_id=$1 AND id=$2 AND deleted_at IS NULL AND kind NOT IN ('credential','portal')",
    )
    .bind(space_id)
    .bind(node_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(db::sql_error)?
    .ok_or("node-unavailable: node does not exist in the requested Space")?;
    let data = parse_json(row.try_get("data").map_err(db::sql_error)?)?;
    let asset_revision = asset_revision(&data)?;
    let chunks = sqlx::query(
        "SELECT c.id,c.kind,c.source_revision,c.text,c.locator,c.updated_at \
         FROM spaces_current_knowledge_chunks c JOIN spaces_semantic_nodes n \
         ON n.id=c.node_id AND n.space_id=c.space_id WHERE c.space_id=$1 AND c.node_id=$2 \
         ORDER BY c.updated_at DESC LIMIT 20",
    )
    .bind(space_id)
    .bind(node_id)
    .fetch_all(&mut *connection)
    .await
    .map_err(db::sql_error)?;
    let chunks = chunks
        .into_iter()
        .map(|chunk| {
            Ok(json!({
                "id": chunk.try_get::<String,_>("id").map_err(db::sql_error)?, "kind": chunk.try_get::<String,_>("kind").map_err(db::sql_error)?,
                "sourceRevision": chunk.try_get::<String,_>("source_revision").map_err(db::sql_error)?,
                "text": chunk.try_get::<String,_>("text").map_err(db::sql_error)?,
                "locator": parse_json(chunk.try_get("locator").map_err(db::sql_error)?)?,
                "updatedAt": chunk.try_get::<i64,_>("updated_at").map_err(db::sql_error)?,
            }))
        })
        .collect::<Result<Vec<Value>, String>>()?;
    Ok(json!({
        "id": row.try_get::<String,_>("id").map_err(db::sql_error)?, "kind": row.try_get::<String,_>("kind").map_err(db::sql_error)?,
        "knowledgeRevision": data.get("knowledgeRevision"), "sourceRevision": data.get("sourceRevision"),
        "assetRevision": asset_revision,
        "data": data, "chunks": chunks, "createdAt": row.try_get::<i64,_>("created_at").map_err(db::sql_error)?,
        "updatedAt": row.try_get::<i64,_>("updated_at").map_err(db::sql_error)?,
    }))
}

fn asset_revision(data: &Value) -> Result<Option<String>, String> {
    if data.get("assetPath").is_none() || data.get("sourceRevision").is_none() {
        return Ok(None);
    }
    let kind = data["kind"]
        .as_str()
        .ok_or("Resource kind is unavailable")?;
    use crate::spaces::derivation::input_revision::{resolve, RevisionBasis};
    let basis = if kind == "image" {
        RevisionBasis::Rendered
    } else {
        RevisionBasis::Asset
    };
    resolve(kind, data, basis).map(|(revision, _)| Some(revision))
}

fn parse_json(raw: String) -> Result<Value, String> {
    serde_json::from_str(&raw).map_err(db::json_error)
}
