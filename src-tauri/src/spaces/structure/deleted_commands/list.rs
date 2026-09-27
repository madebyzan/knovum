use serde::Serialize;
use sqlx::Row;
use tauri::{AppHandle, Runtime};

use crate::spaces::{db, structure::StructureNode};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DeletedStructureRoot {
    batch_id: String,
    root_structure_node_id: String,
    kind: String,
    title: String,
    node_id: Option<String>,
    deleted_at: i64,
    space_id: String,
    space_title: String,
}

pub(crate) async fn list<R: Runtime>(
    app: &AppHandle<R>,
    space_id: Option<&str>,
) -> Result<Vec<DeletedStructureRoot>, String> {
    let pool = db::pool(app).await?;
    let mut tx = pool.begin().await.map_err(db::sql_error)?;
    if let Some(space_id) = space_id {
        let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spaces WHERE id=$1)")
            .bind(space_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(db::sql_error)?;
        if !exists {
            return Err("Space not found".into());
        }
    }
    let rows = sqlx::query(
        "SELECT b.id,b.space_id,b.root_structure_node_ids,b.deleted_at,s.title AS space_title \
         FROM spaces_deleted_structure_batches b JOIN spaces s ON s.id=b.space_id \
         WHERE b.restored_at IS NULL AND ($1 IS NULL OR b.space_id=$1) \
         ORDER BY b.deleted_at DESC,b.id",
    )
    .bind(space_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let mut result = Vec::new();
    for row in rows {
        let batch_id: String = row.try_get("id").map_err(db::sql_error)?;
        let batch_space_id: String = row.try_get("space_id").map_err(db::sql_error)?;
        let space_title: String = row.try_get("space_title").map_err(db::sql_error)?;
        let deleted_at: i64 = row.try_get("deleted_at").map_err(db::sql_error)?;
        let roots: Vec<String> = serde_json::from_str(
            row.try_get::<String, _>("root_structure_node_ids")
                .map_err(db::sql_error)?
                .as_str(),
        )
        .map_err(db::json_error)?;
        for root in roots {
            result.push(
                summarize_root(
                    &mut tx,
                    &batch_id,
                    &batch_space_id,
                    &space_title,
                    deleted_at,
                    root,
                )
                .await?,
            );
        }
    }
    tx.commit().await.map_err(db::sql_error)?;
    Ok(result)
}

async fn summarize_root(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    batch_id: &str,
    space_id: &str,
    space_title: &str,
    deleted_at: i64,
    root_structure_node_id: String,
) -> Result<DeletedStructureRoot, String> {
    let raw: String = sqlx::query_scalar(
        "SELECT data FROM spaces_deleted_structure_nodes WHERE batch_id=$1 AND structure_node_id=$2",
    )
    .bind(batch_id)
    .bind(&root_structure_node_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let structure_node: StructureNode = serde_json::from_str(&raw).map_err(db::json_error)?;
    let (node_id, title) = match &structure_node {
        StructureNode::Content { node_id, .. } => {
            let raw: Option<String> =
                sqlx::query_scalar("SELECT data FROM spaces_nodes WHERE id=$1")
                    .bind(node_id)
                    .fetch_optional(&mut **tx)
                    .await
                    .map_err(db::sql_error)?;
            (Some(node_id.clone()), node_title(raw))
        }
        StructureNode::Group { title, .. } | StructureNode::Section { title, .. } => {
            (None, title.clone())
        }
    };
    Ok(DeletedStructureRoot {
        batch_id: batch_id.to_owned(),
        root_structure_node_id,
        kind: structure_node.kind_name().to_owned(),
        title,
        node_id,
        deleted_at,
        space_id: space_id.to_owned(),
        space_title: space_title.to_owned(),
    })
}

fn node_title(raw: Option<String>) -> String {
    raw.and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .and_then(|data| {
            data.get("title")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "Untitled".into())
}
