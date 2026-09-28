use std::collections::HashMap;

use super::super::repository::SpaceStructureSnapshot;
use super::super::{SizingMode, StructureLayout};

pub(super) type ContentSizeStates = HashMap<String, (f64, f64, SizingMode)>;

pub(super) fn content_size_states<'a>(
    snapshots: impl Iterator<Item = &'a SpaceStructureSnapshot>,
) -> ContentSizeStates {
    snapshots
        .flat_map(|snapshot| &snapshot.layouts)
        .filter_map(|layout| match layout {
            StructureLayout::Content {
                structure_node_id,
                size,
                sizing_mode,
                ..
            } => Some((
                structure_node_id.clone(),
                (size.width, size.height, *sizing_mode),
            )),
            _ => None,
        })
        .collect()
}

pub(super) fn preserve_content_sizes(
    target: &mut SpaceStructureSnapshot,
    current: &ContentSizeStates,
    before: &ContentSizeStates,
    after: &ContentSizeStates,
) {
    for layout in &mut target.layouts {
        let StructureLayout::Content {
            structure_node_id,
            size,
            sizing_mode,
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
            size.width = current.0;
        }
        if before.1 == after.1 {
            size.height = current.1;
        }
        if before.2 == after.2 {
            *sizing_mode = current.2;
        }
    }
}
