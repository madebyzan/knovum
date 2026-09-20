use crate::spaces::{canvas_state::SpaceEditorDocument, db};
pub(in crate::spaces) async fn synchronize(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, document: &SpaceEditorDocument) -> Result<(), String> {
    sqlx::query("DELETE FROM spaces_canvas_asset_references WHERE space_id=$1").bind(space_id).execute(&mut **tx).await.map_err(db::sql_error)?;
    for layout in &document.layout.items {
        let Some(path) = layout.appearance().and_then(|value| value.background.as_deref()) else { continue; };
        if path.trim().is_empty() { return Err("A background resource path cannot be empty".into()); }
        sqlx::query("INSERT INTO spaces_canvas_asset_references(space_id,item_id,path) VALUES($1,$2,$3)")
            .bind(space_id).bind(layout.id()).bind(path).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}
