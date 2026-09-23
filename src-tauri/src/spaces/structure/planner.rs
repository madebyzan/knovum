use super::edit_contract::SpaceEditCommand;
use super::planner_delete::Deletion;
use super::repository::SpaceStructureSnapshot;

pub(super) struct PlanResult {
    pub selection: Vec<String>,
    pub deletion: Option<Deletion>,
}

pub(super) fn plan(
    command: &SpaceEditCommand,
    snapshot: &mut SpaceStructureSnapshot,
) -> Result<PlanResult, String> {
    let selection = match command {
        SpaceEditCommand::SectionCreateEmpty {
            section_id,
            title,
            frame,
            root_order,
        } => super::planner_metadata::create_empty(snapshot, section_id, title, frame, *root_order),
        SpaceEditCommand::SectionDraw {
            section_id,
            title,
            frame,
            contained_root_ids,
        } => super::planner_section::draw(snapshot, section_id, title, frame, contained_root_ids),
        SpaceEditCommand::SectionWrapSelection {
            section_id,
            title,
            root_ids,
        } => super::planner_section::wrap(snapshot, section_id, title, root_ids),
        SpaceEditCommand::SectionResizeFrame {
            structure_node_id,
            frame,
        } => super::planner_section::resize(snapshot, structure_node_id, frame),
        SpaceEditCommand::SectionFit { structure_node_id } => {
            super::planner_section::fit(snapshot, structure_node_id)
        }
        SpaceEditCommand::SectionSetCollapsed {
            structure_node_id,
            collapsed,
        } => super::planner_metadata::collapse(snapshot, structure_node_id, *collapsed),
        SpaceEditCommand::SectionDissolve { structure_node_id } => {
            super::planner_section::dissolve(snapshot, std::slice::from_ref(structure_node_id))
        }
        SpaceEditCommand::GroupCreate {
            group_id,
            root_ids,
            title,
        } => super::planner_group::create(snapshot, group_id, root_ids, title),
        SpaceEditCommand::GroupDissolve { root_ids } => {
            super::planner_group::dissolve(snapshot, root_ids)
        }
        SpaceEditCommand::StructureMove { root_ids, delta } => {
            super::planner_movement::move_structure(snapshot, root_ids, delta)
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
            super::planner_movement::move_structure(snapshot, root_ids, delta)?;
            if let Some(section_id) = expand_section_id {
                super::planner_metadata::collapse(snapshot, section_id, false)?;
            }
            super::planner_hierarchy::reparent(snapshot, root_ids, parent_id.as_deref(), *order)
        }
        SpaceEditCommand::StructureTransform {
            root_ids,
            anchor,
            translation,
            rotation_delta,
            scale_factor,
        } => super::planner_movement::transform_structure(
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
        } => super::planner_hierarchy::reparent(snapshot, root_ids, parent_id.as_deref(), *order),
        SpaceEditCommand::StructureAlign { root_ids, edge } => {
            super::planner_alignment::arrange(snapshot, root_ids, Some(edge), None)
        }
        SpaceEditCommand::StructureDistribute { root_ids, axis } => {
            super::planner_alignment::arrange(snapshot, root_ids, None, Some(axis))
        }
        SpaceEditCommand::StructureReorder {
            root_ids,
            direction,
        } => super::planner_arrangement::reorder(snapshot, root_ids, direction),
        SpaceEditCommand::StructureAutoLayout {
            root_ids,
            direction,
            gap,
        } => super::planner_arrangement::auto_layout(snapshot, root_ids, direction, *gap),
        SpaceEditCommand::StructureClone { .. } => {
            Err("structure.clone requires a database transaction".into())
        }
        SpaceEditCommand::StructureDeleteSubtrees { root_ids } => {
            let deletion = super::planner_delete::plan(snapshot, root_ids)?;
            return Ok(PlanResult {
                selection: Vec::new(),
                deletion: Some(deletion),
            });
        }
        SpaceEditCommand::StructureSetLock { root_ids, locked } => {
            super::planner_metadata::set_lock(snapshot, root_ids, *locked)
        }
        SpaceEditCommand::ContainerRename {
            structure_node_id,
            title,
        } => super::planner_metadata::rename(snapshot, structure_node_id, title),
        SpaceEditCommand::ContentUpdateLayout {
            structure_node_id,
            transform,
            size,
            size_authority,
            appearance,
            file_subpath,
        } => super::planner_metadata::update_content(
            snapshot,
            structure_node_id,
            transform,
            size,
            size_authority,
            appearance,
            file_subpath,
        ),
        SpaceEditCommand::ContentUpdateHeights { updates } => {
            super::planner_metadata::update_content_heights(snapshot, updates)
        }
        SpaceEditCommand::RelationSetStyle { relation_id, color } => {
            super::planner_metadata::relation_style(snapshot, relation_id, color.as_ref())
        }
    }?;
    Ok(PlanResult {
        selection,
        deletion: None,
    })
}
