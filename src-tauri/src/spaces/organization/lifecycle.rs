use std::collections::HashSet;
use super::{context::Context, draft::{Draft, title}};
use crate::spaces::canvas_state::contract::StructureKind;
pub(super) fn rename(context: &Context<'_>, draft: &mut Draft, id: &str, name: &str, selection: &[String]) -> Result<Vec<String>, String> {
    if context.item(id)?.kind == StructureKind::Content { return Err("Content titles belong to the content editor".into()); }
    context.editable(&[id.into()])?;
    draft.items.get_mut(id).ok_or("Container is unavailable")?.title = Some(title(name)?);
    Ok(selection.to_vec())
}
pub(super) fn lock(context: &Context<'_>, draft: &mut Draft, ids: &[String], locked: bool) -> Result<Vec<String>, String> {
    let roots = context.roots(ids, false)?;
    for id in &roots {
        if context.ancestors(id)?.iter().any(|item| item.locked) { return Err("Unlock the ancestor before changing a descendant lock".into()); }
        draft.items.get_mut(id).ok_or("Lock target is unavailable")?.locked = locked;
    }
    Ok(roots)
}
pub(super) fn delete(context: &Context<'_>, draft: &mut Draft, ids: &[String], selection: &[String]) -> Result<Vec<String>, String> {
    let roots = context.roots(ids, false)?; context.editable(&roots)?;
    let subtree = context.subtree(&roots)?;
    if subtree.iter().any(|id| context.items[id.as_str()].locked) { return Err("Unlock protected descendants before deleting this operation unit".into()); }
    let parents: HashSet<_> = roots.iter().map(|id| context.items[id.as_str()].parent_id.clone()).collect();
    for id in subtree { draft.remove(&id); }
    draft.normalize(&parents)?; draft.prune_empty_groups()?;
    Ok(selection.iter().filter(|id| draft.items.contains_key(*id)).cloned().collect())
}
