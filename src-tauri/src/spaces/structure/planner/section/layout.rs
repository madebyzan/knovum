use crate::spaces::structure::edit_contract::SpaceRect;
use crate::spaces::structure::planner::bounds::Bounds;
use crate::spaces::structure::planner::tree_policy::Index;
use crate::spaces::structure::repository::SpaceStructureSnapshot;
use crate::spaces::structure::{Point, Size, StructureLayout, StructureNode};
use std::collections::HashMap;

pub(super) fn append_section(
    snapshot: &mut SpaceStructureSnapshot,
    structure_node_id: &str,
    title: &str,
    frame: &SpaceRect,
    order: u32,
) {
    snapshot.structure.push(StructureNode::Section {
        id: structure_node_id.to_owned(),
        parent_id: None,
        order,
        locked: false,
        title: title.to_owned(),
    });
    snapshot.layouts.push(StructureLayout::Section {
        structure_node_id: structure_node_id.to_owned(),
        position: Point {
            x: frame.x,
            y: frame.y,
        },
        size: Size {
            width: frame.width,
            height: frame.height,
        },
        collapsed: false,
        appearance: None,
        auto_layout: false,
        aspect_locked: false,
    });
}
pub(super) fn require_new(index: &Index, structure_node_id: &str) -> Result<(), String> {
    if index
        .position_by_structure_node_id
        .contains_key(structure_node_id)
    {
        Err(format!(
            "Structure node already exists: {structure_node_id}"
        ))
    } else {
        Ok(())
    }
}
pub(super) fn replace_layouts(
    layouts: &mut [StructureLayout],
    layout_by_structure_node_id: &HashMap<String, StructureLayout>,
) {
    for layout in layouts {
        if let Some(update) = layout_by_structure_node_id.get(layout.structure_node_id()) {
            *layout = update.clone();
        }
    }
}
pub(super) fn shift_local(layout: &mut StructureLayout, dx: f64, dy: f64) -> Result<(), String> {
    match layout {
        StructureLayout::Content { transform, .. } | StructureLayout::Group { transform, .. } => {
            transform.x += dx;
            transform.y += dy;
            Ok(())
        }
        StructureLayout::Section { .. } => Err("Section cannot be a Section child".into()),
    }
}
/// A Section's frame clears its content by the title band above and one margin on every other
/// side; an auto-layout Section also spaces its children by that margin.
pub(super) const SECTION_MARGIN: f64 = 32.0;
pub(super) const SECTION_TITLE_CLEARANCE: f64 = 56.0;
pub(super) const SECTION_MINIMUM_WIDTH: f64 = 240.0;
pub(super) const SECTION_MINIMUM_HEIGHT: f64 = 160.0;

pub(super) fn frame_for_bounds(b: Bounds) -> SpaceRect {
    let required_width = b.right - b.left + 2.0 * SECTION_MARGIN;
    let required_height = b.bottom - b.top + SECTION_TITLE_CLEARANCE + SECTION_MARGIN;
    let width = required_width.max(SECTION_MINIMUM_WIDTH);
    let height = required_height.max(SECTION_MINIMUM_HEIGHT);
    SpaceRect {
        x: b.left - SECTION_MARGIN - (width - required_width) / 2.0,
        y: b.top - SECTION_TITLE_CLEARANCE - (height - required_height) / 2.0,
        width,
        height,
    }
}
