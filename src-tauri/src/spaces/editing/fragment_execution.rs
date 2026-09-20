use crate::spaces::{db, canvas_state::{self, SpaceEditorDocument}, nodes::{self, contracts::InsertNode}, organization::EditorFragment, commands::relations::{self, SpaceRelationInput}};

pub(super) async fn stage(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, fragment: &EditorFragment, after: &SpaceEditorDocument, now: i64) -> Result<Vec<InsertNode>, String> {
    let ids: Vec<_> = fragment.structure.items.iter().map(|item| &item.id).collect();
    let archived: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_editor_trash WHERE space_id=$1 AND item_id IN (SELECT value FROM json_each($2))")
        .bind(space_id).bind(serde_json::to_string(&ids).map_err(db::json_error)?).fetch_one(&mut **tx).await.map_err(db::sql_error)?;
    if archived != 0 { return Err("Inserted identities are still owned by Recently Deleted".into()); }
    super::fragment_types::insert(tx, space_id, &fragment.object_types, now).await?;
    let projection = canvas_state::geometry::project(after)?;
    let content: Vec<_> = fragment.nodes.iter().map(|node| {
        let world = projection.world[&node.id];
        InsertNode { id: node.id.clone(), kind: node.kind.clone(), initial_x: world.x, initial_y: world.y, data: node.data.clone(), source_chunks: node.source_chunks.clone() }
    }).collect();
    nodes::stage_nodes_in_transaction(tx, space_id, &content, now).await?;
    for relation in &fragment.relations {
        relations::create_relation_in_transaction(tx, &SpaceRelationInput {
            id: relation.id.clone(), space_id: space_id.to_string(), source_id: relation.source_id.clone(), target_id: relation.target_id.clone(),
            kind: relation.kind.clone(), rationale: relation.rationale.clone(), status: relation.status.clone(), confidence: relation.confidence,
        }, "human", None, now).await?;
    }
    Ok(content)
}
