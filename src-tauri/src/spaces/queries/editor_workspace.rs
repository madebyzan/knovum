use serde::Serialize;
use serde_json::Value;
use crate::spaces::db;
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EditorWorkspaceContent { pub nodes: Vec<Value>, pub relations: Vec<Value> }
pub(in crate::spaces) async fn read_content(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str) -> Result<EditorWorkspaceContent, String> {
    let nodes = sqlx::query("SELECT id,initial_x,initial_y,data FROM spaces_nodes WHERE space_id=$1 AND deleted_at IS NULL ORDER BY created_at,id")
        .bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?.into_iter().map(super::nodes::node_row).collect::<Result<_,_>>()?;
    let relations = sqlx::query("SELECT id,space_id,source_id,target_id,kind,rationale,status,status_before_invalidation,confidence,actor_kind,actor_id,created_at,updated_at FROM spaces_relations WHERE space_id=$1 AND status<>'invalidated' ORDER BY created_at,id")
        .bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?.into_iter().map(super::relations::relation_row).collect::<Result<_,_>>()?;
    Ok(EditorWorkspaceContent { nodes, relations })
}
