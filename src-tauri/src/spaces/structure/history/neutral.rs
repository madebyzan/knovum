use std::collections::HashMap;

use super::super::repository::SpaceStructureSnapshot;
use super::super::{SizeAuthority, StructureLayout};

pub(super) type ContentHeightStates = HashMap<String, (f64, SizeAuthority)>;

pub(super) fn content_height_states<'a>(
    snapshots: impl Iterator<Item = &'a SpaceStructureSnapshot>,
) -> ContentHeightStates {
    snapshots
        .flat_map(|snapshot| &snapshot.layouts)
        .filter_map(|layout| match layout {
            StructureLayout::Content {
                structure_node_id,
                size,
                size_authority,
                ..
            } => Some((structure_node_id.clone(), (size.height, *size_authority))),
            _ => None,
        })
        .collect()
}

pub(super) fn preserve_content_heights(
    target: &mut SpaceStructureSnapshot,
    current: &ContentHeightStates,
    before: &ContentHeightStates,
    after: &ContentHeightStates,
) {
    for layout in &mut target.layouts {
        let StructureLayout::Content {
            structure_node_id,
            size,
            size_authority,
            ..
        } = layout
        else {
            continue;
        };
        let (Some(current), Some(before), Some(after)) = (
            current.get(structure_node_id),
            before.get(structure_node_id),
            after.get(structure_node_id),
        ) else {
            continue;
        };
        if before.0 == after.0 {
            size.height = current.0;
        }
        if before.1 == after.1 {
            *size_authority = current.1;
        }
    }
}
