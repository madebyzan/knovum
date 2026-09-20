use crate::spaces::{canvas_state::SpaceEditorDocument, db};

pub(in crate::spaces) async fn synchronize(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    space_id: &str,
    document: &SpaceEditorDocument,
) -> Result<(), String> {
    sqlx::query(
        "DELETE FROM spaces_canvas_asset_references WHERE space_id=$1 AND node_id IN (\
         SELECT id FROM spaces_nodes WHERE space_id=$1 AND deleted_at IS NULL)",
    )
    .bind(space_id)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;

    for node in &document.nodes {
        let Some(path) = node
            .appearance
            .as_ref()
            .and_then(|appearance| appearance.background.as_deref())
        else {
            continue;
        };
        if path.is_empty() {
            return Err("Canvas background asset path cannot be empty".into());
        }
        sqlx::query(
            "INSERT INTO spaces_canvas_asset_references(space_id,node_id,path) VALUES($1,$2,$3) \
             ON CONFLICT(space_id,node_id) DO UPDATE SET path=excluded.path",
        )
        .bind(space_id)
        .bind(&node.node_id)
        .bind(path)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    Ok(())
}
