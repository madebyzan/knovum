use std::collections::HashMap;

use super::super::repository::SpaceStructureSnapshot;
use super::super::{SizingMode, StructureLayout};

#[derive(Clone, Copy)]
pub(super) struct ContentSizeState {
    width: f64,
    height: f64,
    sizing_mode: SizingMode,
}

pub(super) type ContentSizeStates = HashMap<String, ContentSizeState>;

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
                ContentSizeState {
                    width: size.width,
                    height: size.height,
                    sizing_mode: *sizing_mode,
                },
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
        if before.width == after.width {
            size.width = current.width;
        }
        if before.height == after.height {
            size.height = current.height;
        }
        if before.sizing_mode == after.sizing_mode {
            *sizing_mode = current.sizing_mode;
        }
    }
}
