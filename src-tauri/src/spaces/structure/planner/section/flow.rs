//! An auto-layout Section owns where its children sit: they fill the Section's width in reading
//! order, as many to a row as fit, and the Section's height follows the rows. Every write settles
//! it (`repository::save_in_transaction`), so adding, removing, resizing or dragging a child, or
//! changing the Section's width, leaves the same packed result however the snapshot changed.

use crate::spaces::structure::planner::bounds::root_bounds;
use crate::spaces::structure::planner::packing::{place, reading_order, widest_table_within};
use crate::spaces::structure::planner::section::layout::{
    SECTION_MARGIN, SECTION_MINIMUM_HEIGHT, SECTION_TITLE_CLEARANCE,
};
use crate::spaces::structure::planner::tree_policy::Index;
use crate::spaces::structure::repository::SpaceStructureSnapshot;
use crate::spaces::structure::{Size, StructureLayout};

/// Align, distribute and auto-layout put roots where the person asked. An auto-layout Section's
/// children are placed by the Section, which would undo the request or reorder by it on settling.
pub(in crate::spaces::structure) fn require_freely_arranged(
    snapshot: &SpaceStructureSnapshot,
    index: &Index,
    roots: &[String],
) -> Result<(), String> {
    for id in roots {
        let Some(parent) = index.node(&snapshot.structure, id)?.parent_id() else {
            continue;
        };
        if matches!(
            snapshot.layouts[index.layout_index_by_structure_node_id[parent]],
            StructureLayout::Section {
                auto_layout: true,
                ..
            }
        ) {
            return Err(format!("An auto-layout Section arranges {id}"));
        }
    }
    Ok(())
}

/// Settles every auto-layout Section; reports whether there was one to settle.
pub(in crate::spaces::structure) fn settle(
    snapshot: &mut SpaceStructureSnapshot,
) -> Result<bool, String> {
    let sections = snapshot
        .layouts
        .iter()
        .filter_map(|layout| match layout {
            StructureLayout::Section {
                structure_node_id,
                auto_layout: true,
                ..
            } => Some(structure_node_id.clone()),
            _ => None,
        })
        .collect::<Vec<_>>();
    if sections.is_empty() {
        return Ok(false);
    }
    let index = Index::new(&snapshot.structure, &snapshot.layouts);
    for section in &sections {
        settle_section(snapshot, &index, section)?;
    }
    Ok(true)
}

fn settle_section(
    snapshot: &mut SpaceStructureSnapshot,
    index: &Index,
    section: &str,
) -> Result<(), String> {
    let children = index
        .children_by_parent_id
        .get(&Some(section.to_owned()))
        .cloned()
        .unwrap_or_default();
    if children.is_empty() {
        return Ok(());
    }
    let layout = index.layout_index_by_structure_node_id[section];
    let StructureLayout::Section { position, size, .. } = &snapshot.layouts[layout] else {
        return Err(format!("Section layout missing: {section}"));
    };
    let (origin, width) = (
        (
            position.x + SECTION_MARGIN,
            position.y + SECTION_TITLE_CLEARANCE,
        ),
        size.width,
    );
    let bounds = children
        .iter()
        .map(|id| root_bounds(&snapshot.structure, &snapshot.layouts, index, id))
        .collect::<Result<Vec<_>, String>>()?;
    let order = reading_order(&bounds);
    let sizes = order
        .iter()
        .map(|child| {
            let bounds = &bounds[*child];
            (bounds.right - bounds.left, bounds.bottom - bounds.top)
        })
        .collect::<Vec<_>>();
    let packed = widest_table_within(&sizes, SECTION_MARGIN, width - 2.0 * SECTION_MARGIN);
    place(snapshot, index, &children, &bounds, &order, &packed, origin)?;
    let StructureLayout::Section { size, .. } = &mut snapshot.layouts[layout] else {
        return Err(format!("Section layout missing: {section}"));
    };
    *size = Size {
        width: width.max(packed.width + 2.0 * SECTION_MARGIN),
        height: (SECTION_TITLE_CLEARANCE + packed.height + SECTION_MARGIN)
            .max(SECTION_MINIMUM_HEIGHT),
    };
    Ok(())
}
