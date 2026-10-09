use std::collections::HashSet;

use crate::spaces::structure::edit_contract::{ContentHeightUpdate, FieldUpdate};
use crate::spaces::structure::planner::tree_policy::{require_editable, Index};
use crate::spaces::structure::repository::SpaceStructureSnapshot;
use crate::spaces::structure::{
    LocalTransform, RelationColor, RelationStyle, Size, SizingMode, StructureLayout,
};
use serde_json::Value;

pub(in crate::spaces::structure) fn update_content(
    snapshot: &mut SpaceStructureSnapshot,
    id: &str,
    transform: &Option<LocalTransform>,
    size: &Option<Size>,
    sizing_mode: &Option<SizingMode>,
    appearance: &FieldUpdate<Value>,
    subpath: &FieldUpdate<String>,
) -> Result<Vec<String>, String> {
    let index = Index::new(&snapshot.structure, &snapshot.layouts);
    if index.node(&snapshot.structure, id)?.kind_name() != "content" {
        return Err(format!("Content structure_node not found: {id}"));
    }
    require_editable(&snapshot.structure, &index, id)?;
    let Some(StructureLayout::Content {
        transform: current_transform,
        size: current_size,
        sizing_mode: current_mode,
        appearance: current_appearance,
        file_subpath: current_subpath,
        ..
    }) = index
        .layout
        .get(id)
        .and_then(|position| snapshot.layouts.get_mut(*position))
    else {
        return Err(format!("Content layout missing: {id}"));
    };
    if let Some(transform) = transform {
        *current_transform = transform.clone();
    }
    if let Some(size) = size {
        *current_size = size.clone();
    }
    if let Some(mode) = sizing_mode {
        *current_mode = *mode;
    }
    apply_update(current_appearance, appearance);
    apply_update(current_subpath, subpath);
    Ok(vec![id.to_owned()])
}

pub(in crate::spaces::structure) fn update_content_heights(
    snapshot: &mut SpaceStructureSnapshot,
    updates: &[ContentHeightUpdate],
) -> Result<Vec<String>, String> {
    if updates.is_empty() {
        return Err("Content height update requires at least one structure_node".into());
    }
    if updates.len() > 100_000 {
        return Err("Content height update exceeds the supported limit".into());
    }
    let index = Index::new(&snapshot.structure, &snapshot.layouts);
    let mut unique_ids = HashSet::with_capacity(updates.len());
    for update in updates {
        if !unique_ids.insert(update.structure_node_id.as_str()) {
            return Err(format!(
                "Duplicate content height update: {}",
                update.structure_node_id
            ));
        }
        if !update.height.is_finite() || update.height <= 0.0 {
            return Err(format!(
                "Invalid content height: {}",
                update.structure_node_id
            ));
        }
        if index
            .node(&snapshot.structure, &update.structure_node_id)?
            .kind_name()
            != "content"
        {
            return Err(format!(
                "Content structure_node not found: {}",
                update.structure_node_id
            ));
        }
        require_editable(&snapshot.structure, &index, &update.structure_node_id)?;
    }
    let mut updated = Vec::with_capacity(updates.len());
    for update in updates {
        let Some(StructureLayout::Content {
            size,
            sizing_mode,
            appearance,
            ..
        }) = index
            .layout
            .get(&update.structure_node_id)
            .and_then(|position| snapshot.layouts.get_mut(*position))
        else {
            return Err(format!(
                "Content layout missing: {}",
                update.structure_node_id
            ));
        };
        let typography_scale = appearance
            .as_ref()
            .and_then(|value| value.get("typographyScale"))
            .and_then(Value::as_f64)
            .unwrap_or(1.0);
        if size.width != update.measured_width
            || *sizing_mode != update.sizing_mode
            || typography_scale != update.typography_scale
        {
            return Err(format!(
                "Content height measurement is stale: {}",
                update.structure_node_id
            ));
        }
        size.height = update.height;
        *sizing_mode = SizingMode::Frame;
        updated.push(update.structure_node_id.clone());
    }
    Ok(updated)
}

pub(in crate::spaces::structure) fn relation_style(
    snapshot: &mut SpaceStructureSnapshot,
    id: &str,
    color: Option<&RelationColor>,
) -> Result<Vec<String>, String> {
    snapshot
        .relation_styles
        .retain(|style| style.relation_id != id);
    if let Some(color) = color {
        snapshot.relation_styles.push(RelationStyle {
            relation_id: id.to_owned(),
            color: color.clone(),
        });
    }
    Ok(Vec::new())
}

fn apply_update<T: Clone>(target: &mut Option<T>, update: &FieldUpdate<T>) {
    match update {
        FieldUpdate::Keep => {}
        FieldUpdate::Set { value } => *target = Some(value.clone()),
        FieldUpdate::Clear => *target = None,
    }
}
