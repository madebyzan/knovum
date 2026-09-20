use super::{contract::StructureCommand, context::Context, draft::Draft, grouping, section_creation, section_editing, reparent, movement, arrangement, lifecycle};
use crate::spaces::canvas_state::{self, contract::{SpaceEditorDocument, StructurePoint}};
pub(in crate::spaces) struct Plan { pub after: SpaceEditorDocument, pub selection: Vec<String>, pub removed_content_ids: Vec<String> }
/** The database owner invokes this planner on its locked authoritative snapshot. */
pub(in crate::spaces) fn plan(before: &SpaceEditorDocument, command: &StructureCommand, selection: &[String]) -> Result<Plan, String> {
    let context = Context::new(before)?;
    for id in selection { context.item(id)?; }
    let mut draft = Draft::new(before);
    let selected = execute(&context, &mut draft, command, selection)?;
    let removed_content_ids = before.structure.items.iter().filter(|item| !draft.items.contains_key(&item.id)).filter_map(|item| item.node_id.clone()).collect();
    let after = draft.finish(before); canvas_state::validate_document(&after)?;
    Ok(Plan { after, selection: selected, removed_content_ids })
}
fn execute(context: &Context<'_>, draft: &mut Draft, command: &StructureCommand, selection: &[String]) -> Result<Vec<String>, String> {
    match command {
        StructureCommand::CreateEmptySection { id, title, bounds } => section_creation::create(context, draft, id, title, *bounds, &[]),
        StructureCommand::DrawSection { id, title, bounds, ids } => section_creation::draw(context, draft, id, title, *bounds, ids),
        StructureCommand::SectionAroundSelection { id, title, ids } => section_creation::wrap(context, draft, id, title, ids),
        StructureCommand::GroupSelection { id, title, ids } => grouping::group(context, draft, ids, id, title),
        StructureCommand::DissolveContainer { id } => grouping::dissolve(context, draft, id),
        StructureCommand::MoveItems { ids, delta } => movement::move_items(context, draft, ids, *delta),
        StructureCommand::ReparentItems { ids, parent_id, insert_index } => reparent::reparent(context, draft, ids, parent_id.as_deref(), *insert_index, StructurePoint { x: 0.0, y: 0.0 }, false),
        StructureCommand::DropItems { ids, parent_id, delta } => reparent::reparent(context, draft, ids, parent_id.as_deref(), None, *delta, true),
        StructureCommand::ResizeSection { id, bounds } => section_editing::resize(context, draft, id, *bounds),
        StructureCommand::FitSection { id } => section_editing::fit(context, draft, id),
        StructureCommand::SetSectionCollapsed { id, collapsed } => section_editing::collapse(context, draft, id, *collapsed, selection),
        StructureCommand::RenameContainer { id, title } => lifecycle::rename(context, draft, id, title, selection),
        StructureCommand::SetItemsLocked { ids, locked } => lifecycle::lock(context, draft, ids, *locked),
        StructureCommand::DeleteSubtrees { ids } => lifecycle::delete(context, draft, ids, selection),
        StructureCommand::TransformGroup { id, world_transform } => movement::transform_group(context, draft, id, *world_transform),
        StructureCommand::AlignItems { ids, axis, alignment } => arrangement::align(context, draft, ids, *axis, *alignment),
        StructureCommand::DistributeItems { ids, axis } => arrangement::distribute(context, draft, ids, *axis),
    }
}
