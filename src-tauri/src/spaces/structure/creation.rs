use crate::spaces::nodes::contracts::InsertNode;

use super::planner_spatial::{local_from_world, world_transform};
use super::planner_support::{require_editable, Index};
use super::repository::SpaceStructureSnapshot;
use super::{LocalTransform, Size, StructureLayout, StructureNode};

pub(crate) fn append_content(
    snapshot: &mut SpaceStructureSnapshot,
    nodes: &[InsertNode],
    parent_id: Option<&str>,
) -> Result<Vec<String>, String> {
    let index = Index::new(&snapshot.structure, &snapshot.layouts);
    let parent_world = match parent_id {
        Some(parent) => {
            let structure_node = index.node(&snapshot.structure, parent)?;
            if structure_node.kind_name() == "content" {
                return Err("Content cannot contain created nodes".into());
            }
            require_editable(&snapshot.structure, &index, parent)?;
            world_transform(&snapshot.structure, &snapshot.layouts, &index, parent)?
        }
        None => identity(),
    };
    let first_order = snapshot
        .structure
        .iter()
        .filter(|structure_node| structure_node.parent_id() == parent_id)
        .count();
    let mut structure_node_ids = Vec::with_capacity(nodes.len());
    for (offset, node) in nodes.iter().enumerate() {
        let structure_node_id = uuid::Uuid::new_v4().to_string();
        let order = u32::try_from(first_order + offset).map_err(|_| "Structure order overflow")?;
        snapshot.structure.push(StructureNode::Content {
            id: structure_node_id.clone(),
            parent_id: parent_id.map(str::to_owned),
            order,
            locked: false,
            node_id: node.id.clone(),
        });
        let world = LocalTransform {
            x: node.initial_x,
            y: node.initial_y,
            rotation: 0.0,
            scale: 1.0,
        };
        snapshot.layouts.push(StructureLayout::Content {
            structure_node_id: structure_node_id.clone(),
            transform: local_from_world(&world, &parent_world),
            size: Size {
                width: node.width,
                height: node.height,
            },
            size_authority: node.size_authority,
            appearance: node.appearance.clone(),
            file_subpath: node.file_subpath.clone(),
        });
        structure_node_ids.push(structure_node_id);
    }
    Ok(structure_node_ids)
}

fn identity() -> LocalTransform {
    LocalTransform {
        x: 0.0,
        y: 0.0,
        rotation: 0.0,
        scale: 1.0,
    }
}
