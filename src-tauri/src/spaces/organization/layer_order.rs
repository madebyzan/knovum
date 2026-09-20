use std::collections::HashSet;
use super::{context::Context, draft::Draft, editor_command::LayerMove};
pub(super) fn apply(context: &Context<'_>, draft: &mut Draft, ids: &[String], mode: LayerMove) -> Result<Vec<String>, String> {
    let roots = context.roots(ids, false)?; context.editable(&roots)?;
    let selected: HashSet<_> = roots.iter().map(String::as_str).collect();
    let parents: HashSet<_> = roots.iter().map(|id| context.items[id.as_str()].parent_id.clone()).collect();
    for parent in parents { let next = reordered(draft.siblings(parent.as_deref()), &selected, mode); draft.set_siblings(parent.as_deref(), &next)?; }
    Ok(roots)
}
fn reordered(mut ids: Vec<String>, selected: &HashSet<&str>, mode: LayerMove) -> Vec<String> {
    match mode {
        LayerMove::Front => { ids.sort_by_key(|id| selected.contains(id.as_str())); },
        LayerMove::Back => { ids.sort_by_key(|id| !selected.contains(id.as_str())); },
        LayerMove::Forward => { for index in (0..ids.len().saturating_sub(1)).rev() { if selected.contains(ids[index].as_str()) && !selected.contains(ids[index+1].as_str()) { ids.swap(index,index+1); } } },
        LayerMove::Backward => { for index in 1..ids.len() { if selected.contains(ids[index].as_str()) && !selected.contains(ids[index-1].as_str()) { ids.swap(index,index-1); } } },
    }
    ids
}
