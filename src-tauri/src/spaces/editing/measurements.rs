use sqlx::Row;
use serde_json::Value;
use crate::spaces::{db, organization::TextMeasurement, canvas_state::contract::{SpaceEditorDocument, StructureKind, StructureLayout}};
pub(super) async fn apply(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, before: &SpaceEditorDocument, measurements: &[TextMeasurement]) -> Result<SpaceEditorDocument, String> {
    if measurements.len() > 10_000 { return Err("Too many content measurements".into()); }
    let mut after = before.clone();
    let mut seen = std::collections::HashSet::new();
    for measurement in measurements {
        if !seen.insert(&measurement.item_id) || !measurement.width.is_finite() || !measurement.height.is_finite() || measurement.width <= 0.0 || measurement.height <= 0.0 { return Err("Invalid content measurement".into()); }
        let item = before.structure.items.iter().find(|item| item.id == measurement.item_id);
        let Some(item) = item else { continue; };
        if item.kind != StructureKind::Content { return Err("Only content can report a text measurement".into()); }
        let row = sqlx::query("SELECT kind,data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL").bind(item.node_id.as_deref()).bind(space_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?;
        let Some(row) = row else { continue; };
        if row.try_get::<String,_>("kind").map_err(db::sql_error)? != "text" { return Err("Only Text uses content-fit height".into()); }
        let data: Value = serde_json::from_str(&row.try_get::<String,_>("data").map_err(db::sql_error)?).map_err(db::json_error)?;
        if crate::spaces::knowledge::indexing::revision::render_revision(&data) != measurement.content_revision { continue; }
        if let Some(StructureLayout::Content { size, appearance, .. }) = after.layout.items.iter_mut().find(|layout| layout.id() == item.id) {
            let scale = appearance.as_ref().and_then(|appearance| appearance.typography_scale).unwrap_or(1.0);
            if (size.width - measurement.width).abs() > 0.5 || scale != measurement.typography_scale { continue; }
            size.height = measurement.height.ceil();
        }
    }
    crate::spaces::canvas_state::validate_document(&after)?;
    Ok(after)
}
