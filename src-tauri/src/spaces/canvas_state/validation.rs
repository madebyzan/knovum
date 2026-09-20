use std::collections::HashSet;
use super::contract::SpaceEditorDocument;
use crate::spaces::db;
pub(in crate::spaces) fn validate_document(document: &SpaceEditorDocument) -> Result<(), String> {
    if document.version != 1 { return Err("Unsupported Space editor document version".into()); }
    super::structure_validation::validate(&document.structure)?;
    super::layout_validation::validate(document)?;
    if !document.viewport.x.is_finite() || !document.viewport.y.is_finite() || !document.viewport.zoom.is_finite() || document.viewport.zoom <= 0.0 { return Err("Invalid camera state".into()); }
    let mut ids = HashSet::new();
    for style in document.relation_styles.iter().flatten() {
        if !super::structure_validation::valid_id(&style.relation_id) || !ids.insert(style.relation_id.as_str()) { return Err("Relation styles must have unique valid identities".into()); }
    }
    if ids.len() > 100_000 { return Err("Too many relation styles".into()); }
    Ok(())
}
pub(in crate::spaces) async fn validate_content_references(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, document: &SpaceEditorDocument) -> Result<(), String> {
    let active: Vec<String> = sqlx::query_scalar("SELECT id FROM spaces_nodes WHERE space_id=$1 AND deleted_at IS NULL").bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    let expected: HashSet<&str> = active.iter().map(String::as_str).collect();
    let actual: HashSet<&str> = document.structure.items.iter().filter_map(|item| item.node_id.as_deref()).collect();
    if expected != actual { return Err("The editor must place exactly the active content belonging to its Space".into()); }
    Ok(())
}
