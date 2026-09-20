use std::collections::{BTreeMap, HashSet};
use super::{changes::RecordChange, lifecycle_state::{LifecycleState, LifecycleChanges, NodeActivity, RelationState, ProvenanceState}};
pub(super) fn direction(changes: &LifecycleChanges, inverse: bool) -> LifecycleChanges {
    if !inverse { return changes.clone(); }
    LifecycleChanges { nodes: reverse(&changes.nodes), relations: reverse(&changes.relations), provenance: reverse(&changes.provenance) }
}
fn reverse<T: Clone>(changes: &[RecordChange<T>]) -> Vec<RecordChange<T>> {
    changes.iter().map(|change| RecordChange { id: change.id.clone(), before: change.after.clone(), after: change.before.clone() }).collect()
}
pub(super) fn validate_before(current: &LifecycleState, changes: &LifecycleChanges) -> Result<(), String> {
    validate(&current.nodes, &changes.nodes, |value| !value.active, equivalent_node)?;
    validate(&current.relations, &changes.relations, |value| value.status == "invalidated", equivalent_relation)?;
    validate(&current.provenance, &changes.provenance, |value| value.status == "invalidated", equivalent_provenance)
}
fn validate<T: PartialEq>(current: &BTreeMap<String,T>, changes: &[RecordChange<T>], inactive: impl Fn(&T) -> bool, same_identity: impl Fn(&T,&T) -> bool) -> Result<(), String> {
    let mut seen = HashSet::new();
    for change in changes {
        if !seen.insert(&change.id) { return Err("Duplicate lifecycle receipt identity".into()); }
        let value = current.get(&change.id).ok_or_else(|| format!("{} was permanently removed and cannot be restored",change.id))?;
        if let Some(before) = &change.before {
            if value != before { return Err(format!("{} changed after this edit; history cannot overwrite it",change.id)); }
        } else if !inactive(value) || change.after.as_ref().is_none_or(|after| !same_identity(value,after)) {
            return Err(format!("{} is no longer a recoverable identity from this edit",change.id));
        }
    }
    Ok(())
}
pub(super) fn validate_after(before: &LifecycleState, after: &LifecycleState, changes: &LifecycleChanges) -> Result<(), String> {
    check_after(&before.nodes, &after.nodes, &changes.nodes, |value| !value.active)?;
    check_after(&before.relations, &after.relations, &changes.relations, |value| value.status == "invalidated")?;
    check_after(&before.provenance, &after.provenance, &changes.provenance, |value| value.status == "invalidated")
}
fn check_after<T: PartialEq>(before: &BTreeMap<String,T>, after: &BTreeMap<String,T>, changes: &[RecordChange<T>], inactive: impl Fn(&T) -> bool) -> Result<(), String> {
    let expected: HashSet<&str> = changes.iter().map(|change| change.id.as_str()).collect();
    for (id,value) in after {
        if before.get(id) != Some(value) && !expected.contains(id.as_str()) { return Err("History would invalidate a dependency created after the original edit".into()); }
    }
    for change in changes {
        let value = after.get(&change.id).ok_or("A lifecycle record disappeared during replay")?;
        if let Some(expected) = &change.after { if value != expected { return Err("Lifecycle replay did not produce the exact committed state".into()); } }
        else if !inactive(value) { return Err("An undone creation remains active".into()); }
    }
    Ok(())
}
fn equivalent_node(a: &NodeActivity, b: &NodeActivity) -> bool { a.id == b.id }
fn equivalent_relation(a: &RelationState, b: &RelationState) -> bool { a.id == b.id && a.source_id == b.source_id && a.target_id == b.target_id && a.kind == b.kind && a.rationale == b.rationale && a.confidence == b.confidence }
fn equivalent_provenance(a: &ProvenanceState, b: &ProvenanceState) -> bool { a.id == b.id && a.target_kind == b.target_kind && a.target_id == b.target_id && a.source_node_id == b.source_node_id && a.source_revision == b.source_revision && a.locator == b.locator && a.quote == b.quote }
