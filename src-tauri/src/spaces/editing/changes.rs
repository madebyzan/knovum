use std::collections::{BTreeMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::spaces::canvas_state::contract::{SpaceEditorDocument, StructureItem, StructureLayout, CanvasRelationStyleV1};
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::spaces) struct RecordChange<T> { pub id: String, pub before: Option<T>, pub after: Option<T> }
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::spaces) struct EditChanges {
    pub version: u8,
    pub items: Vec<RecordChange<StructureItem>>,
    pub layouts: Vec<RecordChange<StructureLayout>>,
    pub relation_styles: Vec<RecordChange<CanvasRelationStyleV1>>,
}
pub(in crate::spaces) fn diff(before: &SpaceEditorDocument, after: &SpaceEditorDocument) -> EditChanges {
    EditChanges { version: 1,
        items: differences(&before.structure.items, &after.structure.items, |item| item.id.as_str()),
        layouts: differences(&before.layout.items, &after.layout.items, |item| item.id()),
        relation_styles: differences(before.relation_styles.as_deref().unwrap_or_default(), after.relation_styles.as_deref().unwrap_or_default(), |style| style.relation_id.as_str()),
    }
}
pub(in crate::spaces) fn apply(current: &SpaceEditorDocument, changes: &EditChanges, inverse: bool) -> Result<SpaceEditorDocument, String> {
    if changes.version != 1 { return Err("Unsupported edit receipt version".into()); }
    let mut after = current.clone();
    after.structure.items = apply_records(&current.structure.items, &changes.items, inverse, |item| item.id.as_str())?;
    after.layout.items = apply_records(&current.layout.items, &changes.layouts, inverse, |item| item.id())?;
    let styles = apply_records(current.relation_styles.as_deref().unwrap_or_default(), &changes.relation_styles, inverse, |style| style.relation_id.as_str())?;
    after.relation_styles = if styles.is_empty() { None } else { Some(styles) };
    crate::spaces::canvas_state::validate_document(&after)?;
    Ok(after)
}
pub(in crate::spaces) fn unchanged(changes: &EditChanges) -> bool { changes.items.is_empty() && changes.layouts.is_empty() && changes.relation_styles.is_empty() }
fn differences<T: Clone + PartialEq>(before: &[T], after: &[T], id: impl Fn(&T) -> &str) -> Vec<RecordChange<T>> {
    let left: BTreeMap<_,_> = before.iter().map(|item| (id(item).to_string(), item)).collect();
    let right: BTreeMap<_,_> = after.iter().map(|item| (id(item).to_string(), item)).collect();
    let keys: std::collections::BTreeSet<_> = left.keys().chain(right.keys()).cloned().collect();
    keys.into_iter().filter_map(|key| {
        let before = left.get(&key).copied(); let after = right.get(&key).copied();
        if before == after { None } else { Some(RecordChange { id: key, before: before.cloned(), after: after.cloned() }) }
    }).collect()
}
fn apply_records<T: Clone + PartialEq>(current: &[T], changes: &[RecordChange<T>], inverse: bool, id: impl Fn(&T) -> &str) -> Result<Vec<T>, String> {
    if changes.len() > 100_000 { return Err("Edit receipt exceeds the record limit".into()); }
    let mut result: BTreeMap<String,T> = current.iter().map(|item| (id(item).to_string(), item.clone())).collect();
    let mut seen = HashSet::new();
    for change in changes {
        if !seen.insert(&change.id) || change.before.is_none() && change.after.is_none() { return Err("Invalid edit receipt record identities".into()); }
        if change.before.as_ref().is_some_and(|value| id(value) != change.id) || change.after.as_ref().is_some_and(|value| id(value) != change.id) { return Err("Edit receipt substitutes a record identity".into()); }
        let (before,after) = if inverse { (&change.after,&change.before) } else { (&change.before,&change.after) };
        if result.get(&change.id) != before.as_ref() { return Err(format!("The item {} changed after this edit; history cannot overwrite it",change.id)); }
        if let Some(value) = after { result.insert(change.id.clone(),value.clone()); } else { result.remove(&change.id); }
    }
    Ok(result.into_values().collect())
}
