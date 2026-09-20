use std::collections::HashSet;
use crate::spaces::canvas_state::{self, contract::{SpaceEditorDocument, StructureKind, StructureLayout}, geometry};
use super::{context::Context, fragment_contract::{EditorFragment, FragmentPlacement}, Plan};

pub(super) fn plan(before: &SpaceEditorDocument, fragment: &EditorFragment, placement: &FragmentPlacement) -> Result<Plan, String> {
    validate_fragment(fragment)?;
    let context = Context::new(before)?;
    if let Some(parent) = &placement.parent_id {
        context.editable(std::slice::from_ref(parent))?;
        if context.item(parent)?.kind == StructureKind::Content { return Err("Content cannot contain pasted items".into()); }
    }
    if fragment.structure.items.iter().any(|item| context.items.contains_key(item.id.as_str())) { return Err("Inserted structure identities already exist in the target Space".into()); }
    if !placement.delta.x.is_finite() || !placement.delta.y.is_finite() { return Err("Insertion offset must be finite".into()); }
    let inverse = geometry::inverse(context.parent_world(placement.parent_id.as_deref())?);
    let mut after = before.clone();
    let first = context.children.get(&placement.parent_id.as_deref()).map_or(0, Vec::len) as u32;
    let roots: HashSet<&str> = fragment.structure.items.iter().filter(|item| item.parent_id.is_none()).map(|item| item.id.as_str()).collect();
    let mut selection = Vec::new();
    let mut items = fragment.structure.items.clone();
    items.sort_by_key(|item| item.order);
    for mut item in items {
        if roots.contains(item.id.as_str()) {
            if item.kind == StructureKind::Section && placement.parent_id.is_some() { return Err("Sections can only be inserted at the Space root".into()); }
            selection.push(item.id.clone()); item.parent_id = placement.parent_id.clone(); item.order += first;
        }
        after.structure.items.push(item);
    }
    for mut layout in fragment.layout.items.clone() {
        if roots.contains(layout.id()) {
            match &mut layout {
                StructureLayout::Content { transform, .. } | StructureLayout::Group { transform, .. } => {
                    transform.x += placement.delta.x; transform.y += placement.delta.y;
                    *transform = geometry::compose(inverse, *transform);
                }
                StructureLayout::Section { position, .. } => { position.x += placement.delta.x; position.y += placement.delta.y; }
            }
        }
        after.layout.items.push(layout);
    }
    if !fragment.relation_styles.is_empty() { after.relation_styles.get_or_insert_with(Vec::new).extend(fragment.relation_styles.iter().cloned()); }
    canvas_state::validate_document(&after)?;
    Ok(Plan { after, selection, removed_content_ids: vec![] })
}
fn validate_fragment(fragment: &EditorFragment) -> Result<(), String> {
    if fragment.structure.items.is_empty() || fragment.structure.items.len() > 10_000 || fragment.nodes.len() > 10_000 || fragment.relations.len() > 100_000 || fragment.object_types.len() > 10_000 { return Err("A native fragment must contain 1 to 10,000 structural items".into()); }
    let mut document = SpaceEditorDocument::empty(); document.structure = fragment.structure.clone(); document.layout = fragment.layout.clone(); document.relation_styles = Some(fragment.relation_styles.clone());
    canvas_state::validate_document(&document)?;
    let placed: HashSet<&str> = fragment.structure.items.iter().filter_map(|item| item.node_id.as_deref()).collect();
    let nodes: HashSet<&str> = fragment.nodes.iter().map(|node| node.id.as_str()).collect();
    if placed != nodes || nodes.len() != fragment.nodes.len() { return Err("Fragment content must match its structure exactly".into()); }
    let mut relations = HashSet::new();
    for relation in &fragment.relations {
        if !canvas_state::valid_structure_id(&relation.id) || !relations.insert(relation.id.as_str()) || !nodes.contains(relation.source_id.as_str()) || !nodes.contains(relation.target_id.as_str()) { return Err("Fragment relationships require unique identities and internal endpoints".into()); }
        if !matches!(relation.status.as_str(), "active" | "contested") { return Err("Only current relationships may be inserted".into()); }
    }
    if fragment.relation_styles.iter().any(|style| !relations.contains(style.relation_id.as_str())) { return Err("Fragment relationship appearance must have an internal relationship".into()); }
    let mut types = HashSet::new();
    for definition in &fragment.object_types {
        if !types.insert(definition.id.as_str()) || !fragment.nodes.iter().any(|node| node.data.get("objectTypeId").and_then(serde_json::Value::as_str) == Some(definition.id.as_str())) { return Err("Inserted custom types must be unique and used by the inserted content".into()); }
    }
    Ok(())
}
