use std::collections::HashMap;

use serde_json::Value;

use uuid::Uuid;

use super::super::{ClipboardNode, ClipboardPayload, ClipboardRelation};
use crate::spaces::structure::edit_contract::SpacePoint;
mod mapping;

use crate::spaces::structure::{
    planner::bounds::{merge, root_bounds},
    planner::tree_policy::Index,
    validate_space_snapshot, StructureLayout, StructureNode,
};

pub(crate) struct RemappedPayload {
    pub(crate) structure: Vec<StructureNode>,
    pub(crate) layouts: Vec<StructureLayout>,
    pub(crate) nodes: Vec<ClipboardNode>,
    pub(crate) relations: Vec<ClipboardRelation>,
}

pub(crate) fn remap(
    payload: ClipboardPayload,
    anchor: &SpacePoint,
    fields_by_object_type_id: &HashMap<String, Value>,
) -> Result<RemappedPayload, String> {
    let structure_node_id_by_source_id = payload
        .structure
        .iter()
        .map(|structure_node| (structure_node.id().to_owned(), Uuid::new_v4().to_string()))
        .collect::<HashMap<_, _>>();
    let node_id_by_source_id = payload
        .nodes
        .iter()
        .map(|node| (node.id.clone(), Uuid::new_v4().to_string()))
        .collect::<HashMap<_, _>>();
    let structure = payload
        .structure
        .into_iter()
        .map(|structure_node| {
            mapping::remap_structure_node(
                structure_node,
                &structure_node_id_by_source_id,
                &node_id_by_source_id,
            )
        })
        .collect::<Vec<_>>();
    let mut layouts = payload
        .layouts
        .into_iter()
        .map(|layout| mapping::remap_layout(layout, &structure_node_id_by_source_id))
        .collect::<Vec<_>>();
    shift_to_anchor(&structure, &mut layouts, anchor)?;
    let nodes = payload
        .nodes
        .into_iter()
        .map(|node| {
            remap_node(
                node,
                &node_id_by_source_id,
                &structure_node_id_by_source_id,
                fields_by_object_type_id,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let relations = payload
        .relations
        .into_iter()
        .map(|relation| remap_relation(relation, &node_id_by_source_id))
        .collect();
    validate_space_snapshot(&structure, &layouts)?;
    Ok(RemappedPayload {
        structure,
        layouts,
        nodes,
        relations,
    })
}

fn shift_to_anchor(
    structure: &[StructureNode],
    layouts: &mut [StructureLayout],
    anchor: &SpacePoint,
) -> Result<(), String> {
    let roots = structure
        .iter()
        .filter(|structure_node| structure_node.parent_id().is_none())
        .map(|structure_node| structure_node.id().to_owned())
        .collect::<Vec<_>>();
    let index = Index::new(structure, layouts);
    let mut bounds = None;
    for root in &roots {
        bounds = Some(merge(
            bounds,
            root_bounds(structure, layouts, &index, root)?,
        ));
    }
    let bounds = bounds.ok_or("Clipboard structure has no roots")?;
    for root in &roots {
        shift_root(layouts, root, anchor.x - bounds.left, anchor.y - bounds.top)?;
    }
    Ok(())
}

fn remap_node(
    mut node: ClipboardNode,
    node_id_by_source_id: &HashMap<String, String>,
    structure_node_id_by_source_id: &HashMap<String, String>,
    fields_by_object_type_id: &HashMap<String, Value>,
) -> Result<ClipboardNode, String> {
    let fields = node
        .data
        .get("objectTypeId")
        .and_then(Value::as_str)
        .and_then(|id| fields_by_object_type_id.get(id));
    crate::spaces::structure::node_reference_remap::remap(
        &mut node.data,
        node_id_by_source_id,
        structure_node_id_by_source_id,
        fields,
    )?;
    Ok(ClipboardNode {
        id: node_id_by_source_id[&node.id].clone(),
        data: node.data,
    })
}

fn remap_relation(
    mut relation: ClipboardRelation,
    node_id_by_source_id: &HashMap<String, String>,
) -> ClipboardRelation {
    relation.id = Uuid::new_v4().to_string();
    relation.source_id = node_id_by_source_id[&relation.source_id].clone();
    relation.target_id = node_id_by_source_id[&relation.target_id].clone();
    relation
}

fn shift_root(
    layouts: &mut [StructureLayout],
    structure_node_id: &str,
    dx: f64,
    dy: f64,
) -> Result<(), String> {
    match layouts
        .iter_mut()
        .find(|layout| layout.structure_node_id() == structure_node_id)
        .ok_or("Clipboard root layout missing")?
    {
        StructureLayout::Content { transform, .. } | StructureLayout::Group { transform, .. } => {
            transform.x += dx;
            transform.y += dy;
        }
        StructureLayout::Section { position, .. } => {
            position.x += dx;
            position.y += dy;
        }
    }
    Ok(())
}
