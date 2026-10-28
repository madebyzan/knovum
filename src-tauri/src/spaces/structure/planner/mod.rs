mod alignment;
mod arrangement;
pub(super) mod bounds;
pub(super) mod clone;
pub(super) mod delete;
mod group;
mod hierarchy;
mod metadata;
mod movement;
mod packing;
mod section;
pub(super) mod spatial;
pub(super) mod tree_policy;

use self::delete::Deletion;
pub(super) use self::section::settle as settle_auto_layout_sections;
use crate::spaces::structure::edit_contract::SpaceEditCommand;
use crate::spaces::structure::repository::SpaceStructureSnapshot;

pub(in crate::spaces::structure) struct StructureEditPlan {
    pub selection: Vec<String>,
    pub deletion: Option<Deletion>,
}

pub(in crate::spaces::structure) fn plan(
    command: &SpaceEditCommand,
    snapshot: &mut SpaceStructureSnapshot,
) -> Result<StructureEditPlan, String> {
    let selection = match command {
        SpaceEditCommand::SectionCreateEmpty {
            section_id,
            title,
            frame,
            root_order,
        } => self::metadata::create_empty(snapshot, section_id, title, frame, *root_order),
        SpaceEditCommand::SectionDraw {
            section_id,
            title,
            frame,
            contained_root_ids,
        } => self::section::draw(snapshot, section_id, title, frame, contained_root_ids),
        SpaceEditCommand::SectionWrapSelection {
            section_id,
            title,
            root_ids,
        } => self::section::wrap(snapshot, section_id, title, root_ids),
        SpaceEditCommand::SectionResizeFrame {
            structure_node_id,
            frame,
        } => self::section::resize(snapshot, structure_node_id, frame),
        SpaceEditCommand::SectionFit { structure_node_id } => {
            self::section::fit(snapshot, structure_node_id)
        }
        SpaceEditCommand::SectionSetCollapsed {
            structure_node_id,
            collapsed,
        } => self::metadata::collapse(snapshot, structure_node_id, *collapsed),
        SpaceEditCommand::SectionDissolve { structure_node_id } => {
            self::section::dissolve(snapshot, std::slice::from_ref(structure_node_id))
        }
        SpaceEditCommand::SectionUpdateLayout {
            structure_node_id,
            auto_layout,
            appearance,
        } => self::metadata::update_section(snapshot, structure_node_id, *auto_layout, appearance),
        SpaceEditCommand::GroupCreate {
            group_id,
            root_ids,
            title,
        } => self::group::create(snapshot, group_id, root_ids, title),
        SpaceEditCommand::GroupDissolve { root_ids } => self::group::dissolve(snapshot, root_ids),
        SpaceEditCommand::StructureMove { root_ids, delta } => {
            self::movement::move_structure(snapshot, root_ids, delta)
        }
        SpaceEditCommand::StructureMoveToContainer {
            root_ids,
            delta,
            parent_id,
            order,
            expand_section_id,
        } => {
            if expand_section_id.is_some() && expand_section_id.as_deref() != parent_id.as_deref() {
                return Err("Expanded Section must be the reparent target".into());
            }
            self::movement::move_structure(snapshot, root_ids, delta)?;
            if let Some(section_id) = expand_section_id {
                self::metadata::collapse(snapshot, section_id, false)?;
            }
            self::hierarchy::reparent(snapshot, root_ids, parent_id.as_deref(), *order)
        }
        SpaceEditCommand::StructureTransform {
            root_ids,
            anchor,
            translation,
            rotation_delta,
            scale_factor,
        } => self::movement::transform_structure(
            snapshot,
            root_ids,
            anchor,
            translation,
            *rotation_delta,
            *scale_factor,
        ),
        SpaceEditCommand::StructureReparent {
            root_ids,
            parent_id,
            order,
        } => self::hierarchy::reparent(snapshot, root_ids, parent_id.as_deref(), *order),
        SpaceEditCommand::StructureAlign { root_ids, edge } => {
            self::alignment::arrange(snapshot, root_ids, Some(edge), None)
        }
        SpaceEditCommand::StructureDistribute { root_ids, axis } => {
            self::alignment::arrange(snapshot, root_ids, None, Some(axis))
        }
        SpaceEditCommand::StructureReorder {
            root_ids,
            direction,
        } => self::arrangement::reorder(snapshot, root_ids, direction),
        SpaceEditCommand::StructureAutoLayout {
            root_ids,
            flow,
            gap,
        } => self::arrangement::auto_layout(snapshot, root_ids, *flow, *gap),
        SpaceEditCommand::StructureClone { .. } => {
            Err("structure.clone requires a database transaction".into())
        }
        SpaceEditCommand::StructureDeleteSubtrees { root_ids } => {
            let deletion = self::delete::plan(snapshot, root_ids)?;
            return Ok(StructureEditPlan {
                selection: Vec::new(),
                deletion: Some(deletion),
            });
        }
        SpaceEditCommand::StructureSetLock { root_ids, locked } => {
            self::metadata::set_lock(snapshot, root_ids, *locked)
        }
        SpaceEditCommand::ContainerRename {
            structure_node_id,
            title,
        } => self::metadata::rename(snapshot, structure_node_id, title),
        SpaceEditCommand::ContentUpdateLayout {
            structure_node_id,
            transform,
            size,
            sizing_mode,
            appearance,
            file_subpath,
        } => self::metadata::update_content(
            snapshot,
            structure_node_id,
            transform,
            size,
            sizing_mode,
            appearance,
            file_subpath,
        ),
        SpaceEditCommand::ContentUpdateHeights { updates } => {
            self::metadata::update_content_heights(snapshot, updates)
        }
        SpaceEditCommand::RelationSetStyle { relation_id, color } => {
            self::metadata::relation_style(snapshot, relation_id, color.as_ref())
        }
    }?;
    Ok(StructureEditPlan {
        selection,
        deletion: None,
    })
}
