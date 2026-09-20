use std::collections::{HashMap, HashSet};
use crate::spaces::{db, canvas_state::{self, contract::SpaceEditorDocument}};
pub(super) async fn removed(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, batch_id: &str, before: &SpaceEditorDocument, after: &SpaceEditorDocument, now: i64) -> Result<(), String> {
    let retained: HashSet<&str> = after.structure.items.iter().map(|item| item.id.as_str()).collect();
    let layouts: HashMap<_,_> = before.layout.items.iter().map(|layout| (layout.id(),layout)).collect();
    let projection = canvas_state::geometry::project(before)?;
    for item in &before.structure.items {
        if retained.contains(item.id.as_str()) { continue; }
        sqlx::query("INSERT INTO spaces_editor_trash(space_id,item_id,batch_id,structure,layout,world_transform,deleted_at) VALUES($1,$2,$3,$4,$5,$6,$7) ON CONFLICT(space_id,item_id) DO UPDATE SET batch_id=excluded.batch_id,structure=excluded.structure,layout=excluded.layout,world_transform=excluded.world_transform,deleted_at=excluded.deleted_at")
            .bind(space_id).bind(&item.id).bind(batch_id).bind(serde_json::to_string(item).map_err(db::json_error)?).bind(serde_json::to_string(layouts.get(item.id.as_str()).ok_or("Missing deleted layout")?).map_err(db::json_error)?)
            .bind(serde_json::to_string(projection.world.get(&item.id).ok_or("Missing deleted transform")?).map_err(db::json_error)?).bind(now).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}
pub(super) async fn remove_restored(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, after: &SpaceEditorDocument) -> Result<(), String> {
    let ids: Vec<_> = after.structure.items.iter().map(|item| &item.id).collect();
    sqlx::query("DELETE FROM spaces_editor_trash WHERE space_id=$1 AND item_id IN (SELECT value FROM json_each($2))").bind(space_id).bind(serde_json::to_string(&ids).map_err(db::json_error)?).execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}
