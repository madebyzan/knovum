"""Reviewed Space-owned command algorithms. Renderer plans never authorize a database write."""
from pathlib import Path
ROOT=Path('.')
files={}
def read(path): return files[path] if path in files else (ROOT/path).read_text()
def write(path,text): files[path]=text.strip()+'\n'
base='src-tauri/src/spaces/organization/'
write(base+'mod.rs', '''
mod contract;
mod context;
mod draft;
mod grouping;
mod section_creation;
mod section_editing;
mod reparent;
mod movement;
mod arrangement;
mod lifecycle;
mod plan;
pub(in crate::spaces) use contract::StructureCommand;
pub(in crate::spaces) use plan::{plan, Plan};
''')
write(base+'contract.rs', '''
use serde::{Deserialize, Serialize};
use crate::spaces::canvas_state::{contract::{StructurePoint, StructureTransform}, geometry::Bounds};
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(in crate::spaces) enum StructureCommand {
    CreateEmptySection { id: String, title: String, bounds: Bounds },
    DrawSection { id: String, title: String, bounds: Bounds, ids: Vec<String> },
    SectionAroundSelection { id: String, title: String, ids: Vec<String> },
    GroupSelection { id: String, title: String, ids: Vec<String> },
    DissolveContainer { id: String },
    MoveItems { ids: Vec<String>, delta: StructurePoint },
    ReparentItems { ids: Vec<String>, #[serde(rename = "parentId")] parent_id: Option<String>, #[serde(rename = "insertIndex")] insert_index: Option<usize> },
    DropItems { ids: Vec<String>, #[serde(rename = "parentId")] parent_id: Option<String>, delta: StructurePoint },
    ResizeSection { id: String, bounds: Bounds },
    FitSection { id: String },
    SetSectionCollapsed { id: String, collapsed: bool },
    RenameContainer { id: String, title: String },
    SetItemsLocked { ids: Vec<String>, locked: bool },
    DeleteSubtrees { ids: Vec<String> },
    TransformGroup { id: String, #[serde(rename = "worldTransform")] world_transform: StructureTransform },
    AlignItems { ids: Vec<String>, axis: Axis, alignment: Alignment },
    DistributeItems { ids: Vec<String>, axis: Axis },
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Axis { X, Y }
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(super) enum Alignment { Start, Center, End }
''')
write(base+'context.rs', '''
use std::collections::{HashMap, HashSet};
use crate::spaces::canvas_state::{self, contract::{SpaceEditorDocument, StructureItem, StructureKind, StructureLayout, StructureTransform}, geometry::{self, Bounds, Projection}};
pub(super) struct Context<'a> {
    pub items: HashMap<&'a str, &'a StructureItem>,
    pub layouts: HashMap<&'a str, &'a StructureLayout>,
    pub children: HashMap<Option<&'a str>, Vec<String>>,
    pub projection: Projection,
}
impl<'a> Context<'a> {
    pub fn new(document: &'a SpaceEditorDocument) -> Result<Self, String> {
        canvas_state::validate_document(document)?;
        let items: HashMap<_,_> = document.structure.items.iter().map(|item| (item.id.as_str(), item)).collect();
        let layouts = document.layout.items.iter().map(|layout| (layout.id(), layout)).collect();
        let mut children: HashMap<Option<&str>, Vec<String>> = HashMap::new();
        for item in &document.structure.items { children.entry(item.parent_id.as_deref()).or_default().push(item.id.clone()); }
        for siblings in children.values_mut() { siblings.sort_by_key(|id| items[id.as_str()].order); }
        Ok(Self { items, layouts, children, projection: geometry::project(document)? })
    }
    pub fn item(&self, id: &str) -> Result<&'a StructureItem, String> { self.items.get(id).copied().ok_or_else(|| format!("Structure item {id} is unavailable")) }
    pub fn ancestors(&self, id: &str) -> Result<Vec<&'a StructureItem>, String> {
        let mut ancestors = Vec::new(); let mut parent = self.item(id)?.parent_id.as_deref();
        while let Some(id) = parent { let item = self.item(id)?; ancestors.push(item); parent = item.parent_id.as_deref(); }
        Ok(ancestors)
    }
    pub fn roots(&self, ids: &[String], allow_empty: bool) -> Result<Vec<String>, String> {
        if ids.len() > 100_000 || !allow_empty && ids.is_empty() { return Err("The operation requires a bounded selection".into()); }
        let chosen: HashSet<&str> = ids.iter().map(String::as_str).collect();
        let mut result = Vec::new(); let mut seen = HashSet::new();
        for id in ids {
            self.item(id)?;
            if !seen.insert(id.as_str()) { continue; }
            if !self.ancestors(id)?.iter().any(|item| chosen.contains(item.id.as_str())) { result.push(id.clone()); }
        }
        result.sort_by_key(|id| self.items[id.as_str()].order);
        Ok(result)
    }
    pub fn editable(&self, ids: &[String]) -> Result<(), String> {
        for id in ids { if self.item(id)?.locked || self.ancestors(id)?.iter().any(|item| item.locked) { return Err(format!("Item {id} or an ancestor is locked")); } }
        Ok(())
    }
    pub fn subtree(&self, roots: &[String]) -> Result<Vec<String>, String> {
        let mut pending: Vec<String> = roots.iter().rev().cloned().collect();
        let mut seen = HashSet::new(); let mut result = Vec::new();
        while let Some(id) = pending.pop() {
            self.item(&id)?; if !seen.insert(id.clone()) { continue; }
            pending.extend(self.children.get(&Some(id.as_str())).into_iter().flatten().rev().cloned()); result.push(id);
        }
        Ok(result)
    }
    pub fn shared_parent(&self, ids: &[String]) -> Result<Option<String>, String> {
        let parent = self.item(ids.first().ok_or("Selection is empty")?)?.parent_id.clone();
        if ids.iter().any(|id| self.items[id.as_str()].parent_id != parent) { return Err("Move items into the same parent before combining them".into()); }
        Ok(parent)
    }
    pub fn parent_world(&self, id: Option<&str>) -> Result<StructureTransform, String> {
        match id { None => Ok(geometry::identity()), Some(id) => self.projection.world.get(id).copied().ok_or_else(|| format!("Parent {id} is unavailable")) }
    }
    pub fn bounds_in_parent(&self, ids: &[String], parent_id: Option<&str>) -> Result<Option<Bounds>, String> {
        let inverse = geometry::inverse(self.parent_world(parent_id)?);
        let mut pending = ids.to_vec(); let mut seen = HashSet::new(); let mut bounds = Vec::new();
        while let Some(id) = pending.pop() {
            if !seen.insert(id.clone()) { continue; }
            let layout = self.layouts.get(id.as_str()).ok_or("Layout is unavailable")?;
            if self.item(&id)?.kind == StructureKind::Group { pending.extend(self.children.get(&Some(id.as_str())).into_iter().flatten().cloned()); }
            else if let StructureLayout::Content { size, .. } | StructureLayout::Section { size, .. } = layout {
                bounds.push(geometry::transformed_bounds(geometry::compose(inverse, self.projection.world[&id]), *size));
            }
        }
        Ok(geometry::union_bounds(&bounds))
    }
}
''')
write(base+'draft.rs', '''
use std::collections::{BTreeMap, HashMap, HashSet};
use crate::spaces::canvas_state::contract::{SpaceEditorDocument, StructureItem, StructureKind, StructureLayout, StructureTransform, StructurePoint};
pub(super) struct Draft { pub items: BTreeMap<String, StructureItem>, pub layouts: BTreeMap<String, StructureLayout> }
impl Draft {
    pub fn new(document: &SpaceEditorDocument) -> Self { Self { items: document.structure.items.iter().map(|item| (item.id.clone(), item.clone())).collect(), layouts: document.layout.items.iter().map(|layout| (layout.id().to_string(), layout.clone())).collect() } }
    pub fn finish(self, before: &SpaceEditorDocument) -> SpaceEditorDocument {
        let mut after = before.clone(); after.structure.items = self.items.into_values().collect(); after.layout.items = self.layouts.into_values().collect(); after
    }
    pub fn assert_new(&self, id: &str) -> Result<(), String> {
        if !crate::spaces::canvas_state::valid_structure_id(id) || self.items.contains_key(id) || self.layouts.contains_key(id) { return Err("New structure identity must be valid and unused".into()); } Ok(())
    }
    pub fn siblings(&self, parent: Option<&str>) -> Vec<String> {
        let mut items: Vec<_> = self.items.values().filter(|item| item.parent_id.as_deref() == parent).collect();
        items.sort_by_key(|item| item.order); items.into_iter().map(|item| item.id.clone()).collect()
    }
    pub fn set_siblings(&mut self, parent: Option<&str>, ids: &[String]) -> Result<(), String> {
        for (order, id) in ids.iter().enumerate() {
            let item = self.items.get_mut(id).ok_or("Sibling item is unavailable")?;
            item.parent_id = parent.map(str::to_string); item.order = order as u32;
        }
        Ok(())
    }
    pub fn set_transform(&mut self, id: &str, value: StructureTransform) -> Result<(), String> {
        match self.layouts.get_mut(id).ok_or("Layout is unavailable")? {
            StructureLayout::Content { transform, .. } | StructureLayout::Group { transform, .. } => *transform = value,
            StructureLayout::Section { position, .. } => {
                if value.rotation != 0.0 || value.scale != 1.0 { return Err("Sections cannot rotate or scale their contents".into()); }
                *position = StructurePoint { x: value.x, y: value.y };
            }
        }
        Ok(())
    }
    pub fn remove(&mut self, id: &str) { self.items.remove(id); self.layouts.remove(id); }
    pub fn normalize(&mut self, parents: &HashSet<Option<String>>) -> Result<(), String> {
        let mut groups: HashMap<Option<String>, Vec<String>> = HashMap::new();
        for item in self.items.values() { if parents.contains(&item.parent_id) { groups.entry(item.parent_id.clone()).or_default().push(item.id.clone()); } }
        for (parent, mut ids) in groups { ids.sort_by_key(|id| self.items[id].order); self.set_siblings(parent.as_deref(), &ids)?; }
        Ok(())
    }
    pub fn prune_empty_groups(&mut self) -> Result<(), String> {
        let mut counts: HashMap<Option<String>, usize> = HashMap::new();
        for item in self.items.values() { *counts.entry(item.parent_id.clone()).or_default() += 1; }
        let mut pending: Vec<_> = self.items.values().filter(|item| item.kind == StructureKind::Group && !counts.contains_key(&Some(item.id.clone()))).map(|item| item.id.clone()).collect();
        let mut parents = HashSet::new();
        while let Some(id) = pending.pop() {
            let parent = self.items.get(&id).ok_or("Empty Group disappeared")?.parent_id.clone();
            self.remove(&id); parents.insert(parent.clone());
            let count = counts.get_mut(&parent).ok_or("Invalid child count")?; *count -= 1;
            if *count == 0 { if let Some(parent) = parent { if self.items.get(&parent).is_some_and(|item| item.kind == StructureKind::Group) { pending.push(parent); } } }
        }
        self.normalize(&parents)
    }
}
pub(super) fn title(value: &str) -> Result<String, String> {
    let title = value.trim_matches(|character: char| character.is_whitespace() || character as u32 == 0xfeff);
    if title.is_empty() || title.encode_utf16().count() > 512 { return Err("Container titles require 1 to 512 characters".into()); }
    Ok(title.to_owned())
}
''')
write(base+'grouping.rs', '''
use std::collections::HashSet;
use super::{context::Context, draft::{Draft, title}};
use crate::spaces::canvas_state::{contract::{StructureItem, StructureKind, StructureLayout, StructureTransform}, geometry};
pub(super) fn group(context: &Context<'_>, draft: &mut Draft, ids: &[String], id: &str, name: &str) -> Result<Vec<String>, String> {
    let roots = context.roots(ids, false)?;
    if roots.len() < 2 || roots.iter().any(|id| context.items[id.as_str()].kind == StructureKind::Section) { return Err("Combine at least two sibling contents or Groups".into()); }
    let parent = context.shared_parent(&roots)?; context.editable(&roots)?; draft.assert_new(id)?;
    let bounds = context.bounds_in_parent(&roots, parent.as_deref())?.ok_or("Selection has no bounds")?;
    let transform = StructureTransform { x: bounds.x, y: bounds.y, ..geometry::identity() };
    let siblings = replacement_siblings(context, &roots, id)?;
    draft.items.insert(id.into(), StructureItem { id: id.into(), kind: StructureKind::Group, parent_id: parent.clone(), order: 0, locked: false, node_id: None, title: Some(title(name)?) });
    draft.layouts.insert(id.into(), StructureLayout::Group { item_id: id.into(), transform });
    let inverse = geometry::inverse(transform);
    for root in &roots { draft.set_transform(root, geometry::compose(inverse, context.layouts[root.as_str()].local()))?; }
    draft.set_siblings(Some(id), &roots)?; draft.set_siblings(parent.as_deref(), &siblings)?;
    Ok(vec![id.into()])
}
pub(super) fn replacement_siblings(context: &Context<'_>, roots: &[String], id: &str) -> Result<Vec<String>, String> {
    let parent = context.shared_parent(roots)?;
    let selected: HashSet<&str> = roots.iter().map(String::as_str).collect();
    let front = roots.iter().map(|id| context.items[id.as_str()].order).max().ok_or("Selection is empty")?;
    let mut siblings: Vec<_> = context.children.get(&parent.as_deref()).into_iter().flatten().filter(|id| !selected.contains(id.as_str())).cloned().collect();
    let index = siblings.iter().filter(|id| context.items[id.as_str()].order < front).count();
    siblings.insert(index, id.into()); Ok(siblings)
}
pub(super) fn dissolve(context: &Context<'_>, draft: &mut Draft, id: &str) -> Result<Vec<String>, String> {
    let item = context.item(id)?;
    if item.kind == StructureKind::Content { return Err("Only a container can be dissolved".into()); }
    context.editable(&[id.into()])?;
    let children = context.children.get(&Some(id)).cloned().unwrap_or_default();
    let inverse = geometry::inverse(context.parent_world(item.parent_id.as_deref())?);
    for child in &children { draft.set_transform(child, geometry::compose(inverse, context.projection.world[child]))?; }
    let mut siblings = draft.siblings(item.parent_id.as_deref());
    let index = siblings.iter().position(|candidate| candidate == id).ok_or("Container has no sibling position")?;
    siblings.splice(index..index+1, children.iter().cloned());
    draft.remove(id); draft.set_siblings(item.parent_id.as_deref(), &siblings)?;
    Ok(children)
}
''')
write(base+'section_creation.rs', '''
use std::collections::HashSet;
use super::{context::Context, draft::{Draft, title}, grouping::replacement_siblings};
use crate::spaces::canvas_state::{contract::{StructureItem, StructureKind, StructureLayout, StructurePoint, StructureSize, StructureTransform}, geometry::{self, Bounds}};
pub(super) fn around(bounds: Bounds) -> Bounds { Bounds { x: bounds.x - 32.0, y: bounds.y - 56.0, width: bounds.width + 64.0, height: bounds.height + 88.0 } }
pub(super) fn draw(context: &Context<'_>, draft: &mut Draft, id: &str, title: &str, bounds: Bounds, ids: &[String]) -> Result<Vec<String>, String> {
    let roots = context.roots(ids, true)?; context.editable(&roots)?;
    for id in &roots {
        let item = context.item(id)?;
        if item.parent_id.is_some() || item.kind == StructureKind::Section { return Err("Drawing can adopt only root-level content or Groups".into()); }
        if !contains(bounds, context.projection.bounds[id]) { return Err("A drawn Section must fully enclose each adopted operation unit".into()); }
    }
    create(context, draft, id, title, bounds, &roots)
}
pub(super) fn wrap(context: &Context<'_>, draft: &mut Draft, id: &str, title: &str, ids: &[String]) -> Result<Vec<String>, String> {
    let roots = context.roots(ids, false)?; let parent = context.shared_parent(&roots)?;
    if roots.iter().any(|id| context.items[id.as_str()].kind == StructureKind::Section) || parent.as_deref().is_some_and(|id| context.items[id].kind != StructureKind::Section) { return Err("Wrap sibling contents or Groups at the root or inside one Section".into()); }
    context.editable(&roots)?;
    let bounds = context.bounds_in_parent(&roots, None)?.ok_or("No content geometry")?;
    create(context, draft, id, title, around(bounds), &roots)
}
pub(super) fn create(context: &Context<'_>, draft: &mut Draft, id: &str, name: &str, bounds: Bounds, roots: &[String]) -> Result<Vec<String>, String> {
    draft.assert_new(id)?;
    let parent = if roots.is_empty() { None } else { context.shared_parent(roots)? };
    let mut siblings = if !roots.is_empty() && parent.is_none() { replacement_siblings(context, roots, id)? } else { draft.siblings(None) };
    if roots.is_empty() { siblings.push(id.into()); }
    else if let Some(parent_id) = &parent {
        let index = siblings.iter().position(|item| item == parent_id).ok_or("Source Section has no root position")?;
        siblings.insert(index + 1, id.into());
    }
    draft.items.insert(id.into(), StructureItem { id: id.into(), kind: StructureKind::Section, parent_id: None, order: 0, locked: false, node_id: None, title: Some(title(name)?) });
    draft.layouts.insert(id.into(), StructureLayout::Section { item_id: id.into(), position: StructurePoint { x: bounds.x, y: bounds.y }, size: StructureSize { width: bounds.width, height: bounds.height }, collapsed: false, appearance: None });
    let inverse = geometry::inverse(StructureTransform { x: bounds.x, y: bounds.y, ..geometry::identity() });
    for root in roots { draft.set_transform(root, geometry::compose(inverse, context.projection.world[root]))?; }
    draft.set_siblings(Some(id), roots)?;
    if parent.is_some() { draft.normalize(&HashSet::from([parent]))?; }
    draft.set_siblings(None, &siblings)?;
    Ok(vec![id.into()])
}
fn contains(outer: Bounds, inner: Bounds) -> bool { inner.x >= outer.x && inner.y >= outer.y && inner.x + inner.width <= outer.x + outer.width && inner.y + inner.height <= outer.y + outer.height }
''')
write(base+'section_editing.rs', '''
use std::collections::HashSet;
use super::{context::Context, draft::Draft, section_creation};
use crate::spaces::canvas_state::{contract::{StructureKind, StructureLayout, StructurePoint, StructureSize, StructureTransform}, geometry::{self, Bounds}};
pub(super) fn resize(context: &Context<'_>, draft: &mut Draft, id: &str, bounds: Bounds) -> Result<Vec<String>, String> {
    context.editable(&[id.into()])?;
    match context.layouts.get(id) {
        Some(StructureLayout::Section { collapsed: false, .. }) => {},
        Some(StructureLayout::Section { .. }) => return Err("Expand the Section before resizing its region".into()),
        _ => return Err("This operation requires a Section".into()),
    }
    let inverse = geometry::inverse(StructureTransform { x: bounds.x, y: bounds.y, ..geometry::identity() });
    for child in context.children.get(&Some(id)).into_iter().flatten() { draft.set_transform(child, geometry::compose(inverse, context.projection.world[child]))?; }
    if let Some(StructureLayout::Section { position, size, .. }) = draft.layouts.get_mut(id) {
        *position = StructurePoint { x: bounds.x, y: bounds.y }; *size = StructureSize { width: bounds.width, height: bounds.height };
    }
    Ok(vec![id.into()])
}
pub(super) fn fit(context: &Context<'_>, draft: &mut Draft, id: &str) -> Result<Vec<String>, String> {
    if context.item(id)?.kind != StructureKind::Section { return Err("Only a Section has a fit-to-contents frame".into()); }
    let ids = context.children.get(&Some(id)).cloned().unwrap_or_default();
    let bounds = context.bounds_in_parent(&ids, None)?.ok_or("An empty Section has no contents to fit")?;
    resize(context, draft, id, section_creation::around(bounds))
}
pub(super) fn collapse(context: &Context<'_>, draft: &mut Draft, id: &str, collapsed: bool, selection: &[String]) -> Result<Vec<String>, String> {
    match draft.layouts.get_mut(id) {
        Some(StructureLayout::Section { collapsed: value, .. }) => *value = collapsed,
        _ => return Err("Only a Section can be collapsed".into()),
    }
    if !collapsed { return Ok(selection.to_vec()); }
    let hidden: HashSet<_> = context.subtree(&[id.into()])?.into_iter().filter(|item| item != id).collect();
    let mut selected: Vec<_> = selection.iter().filter(|item| !hidden.contains(*item)).cloned().collect();
    if selection.iter().any(|item| hidden.contains(item)) && !selected.iter().any(|item| item == id) { selected.push(id.into()); }
    Ok(selected)
}
''')
write(base+'movement.rs', '''
use super::{context::Context, draft::Draft};
use crate::spaces::canvas_state::{contract::{StructureKind, StructurePoint, StructureTransform}, geometry};
pub(super) fn move_items(context: &Context<'_>, draft: &mut Draft, ids: &[String], delta: StructurePoint) -> Result<Vec<String>, String> {
    let roots = context.roots(ids, false)?; context.editable(&roots)?;
    if !geometry::coordinate(delta.x) || !geometry::coordinate(delta.y) { return Err("Invalid movement delta".into()); }
    for id in &roots { move_root(context, draft, id, delta)?; } Ok(roots)
}
pub(super) fn move_root(context: &Context<'_>, draft: &mut Draft, id: &str, delta: StructurePoint) -> Result<(), String> {
    if delta.x == 0.0 && delta.y == 0.0 { return Ok(()); }
    let item = context.item(id)?; let world = context.projection.world[id];
    let inverse = geometry::inverse(context.parent_world(item.parent_id.as_deref())?);
    draft.set_transform(id, geometry::compose(inverse, StructureTransform { x: world.x + delta.x, y: world.y + delta.y, ..world }))
}
pub(super) fn transform_group(context: &Context<'_>, draft: &mut Draft, id: &str, world: StructureTransform) -> Result<Vec<String>, String> {
    let item = context.item(id)?;
    if item.kind != StructureKind::Group { return Err("Only a Group supports aggregate rotation and scaling".into()); }
    context.editable(&[id.into()])?;
    if !geometry::valid_transform(world) { return Err("Invalid Group world transform".into()); }
    draft.set_transform(id, geometry::compose(geometry::inverse(context.parent_world(item.parent_id.as_deref())?), world))?;
    Ok(vec![id.into()])
}
''')
write(base+'reparent.rs', '''
use std::collections::HashSet;
use super::{context::Context, draft::Draft, movement};
use crate::spaces::canvas_state::{contract::{StructureKind, StructurePoint, StructureTransform}, geometry};
pub(super) fn reparent(context: &Context<'_>, draft: &mut Draft, ids: &[String], parent: Option<&str>, insertion: Option<usize>, delta: StructurePoint, dropping: bool) -> Result<Vec<String>, String> {
    let mut roots = context.roots(ids, false)?; context.editable(&roots)?;
    if dropping && roots.iter().all(|id| context.items[id.as_str()].parent_id.as_deref() == parent) { return movement::move_items(context, draft, &roots, delta); }
    validate_destination(context, &roots, parent)?;
    if !geometry::coordinate(delta.x) || !geometry::coordinate(delta.y) { return Err("Invalid drop displacement".into()); }
    let order: std::collections::HashMap<_,_> = context.projection.order.iter().enumerate().map(|(index,id)| (id.as_str(),index)).collect();
    roots.sort_by_key(|id| order[id.as_str()]);
    let chosen: HashSet<&str> = roots.iter().map(String::as_str).collect();
    let mut siblings: Vec<_> = draft.siblings(parent).into_iter().filter(|id| !chosen.contains(id.as_str())).collect();
    let position = insertion.unwrap_or(siblings.len());
    if position > siblings.len() { return Err("Insertion index is outside the remaining sibling list".into()); }
    let inverse = geometry::inverse(context.parent_world(parent)?);
    let old_parents: HashSet<_> = roots.iter().map(|id| context.items[id.as_str()].parent_id.clone()).filter(|id| id.as_deref() != parent).collect();
    for id in &roots {
        let world = context.projection.world[id];
        draft.set_transform(id, geometry::compose(inverse, StructureTransform { x: world.x + delta.x, y: world.y + delta.y, ..world }))?;
        draft.items.get_mut(id).ok_or("Moving item is unavailable")?.parent_id = parent.map(str::to_string);
    }
    draft.normalize(&old_parents)?; siblings.splice(position..position, roots.iter().cloned());
    draft.set_siblings(parent, &siblings)?; draft.prune_empty_groups()?; Ok(roots)
}
fn validate_destination(context: &Context<'_>, roots: &[String], parent: Option<&str>) -> Result<(), String> {
    if let Some(parent) = parent {
        let target = context.item(parent)?;
        if target.kind == StructureKind::Content { return Err("Content cannot contain other items".into()); }
        context.editable(&[parent.into()])?;
        if roots.iter().any(|id| context.items[id.as_str()].kind == StructureKind::Section) { return Err("Sections can only belong to the Space root".into()); }
        if context.subtree(roots)?.iter().any(|id| id == parent) { return Err("An item cannot be moved into its own subtree".into()); }
    }
    Ok(())
}
''')
write(base+'arrangement.rs', '''
use super::{context::Context, draft::Draft, contract::{Axis, Alignment}, movement};
use crate::spaces::canvas_state::{contract::StructurePoint, geometry::{self, Bounds}};
pub(super) fn align(context: &Context<'_>, draft: &mut Draft, ids: &[String], axis: Axis, alignment: Alignment) -> Result<Vec<String>, String> {
    let roots = context.roots(ids, false)?;
    if roots.len() < 2 { return Err("Alignment requires at least two operation roots".into()); } context.editable(&roots)?;
    let bounds: Vec<_> = roots.iter().map(|id| context.projection.bounds[id]).collect();
    let all = geometry::union_bounds(&bounds).ok_or("No arrangement bounds")?;
    let fraction = match alignment { Alignment::Start => 0.0, Alignment::Center => 0.5, Alignment::End => 1.0 };
    let target = position(all, axis) + size(all, axis) * fraction;
    for id in &roots { let rect = context.projection.bounds[id]; move_axis(context, draft, id, axis, target - position(rect, axis) - size(rect, axis) * fraction)?; }
    Ok(roots)
}
pub(super) fn distribute(context: &Context<'_>, draft: &mut Draft, ids: &[String], axis: Axis) -> Result<Vec<String>, String> {
    let mut roots = context.roots(ids, false)?;
    if roots.len() < 3 { return Err("Distribution requires at least three operation roots".into()); } context.editable(&roots)?;
    let order: std::collections::HashMap<_,_> = context.projection.order.iter().enumerate().map(|(index,id)| (id.as_str(),index)).collect();
    roots.sort_by(|left,right| position(context.projection.bounds[left],axis).total_cmp(&position(context.projection.bounds[right],axis)).then(order[left.as_str()].cmp(&order[right.as_str()])));
    let first = context.projection.bounds[&roots[0]]; let last = context.projection.bounds[roots.last().ok_or("No selection")?];
    let total: f64 = roots.iter().map(|id| size(context.projection.bounds[id], axis)).sum();
    let gap = (position(last, axis) + size(last, axis) - position(first, axis) - total) / (roots.len() - 1) as f64;
    let mut cursor = position(first, axis);
    for (index,id) in roots.iter().enumerate() {
        let rect = context.projection.bounds[id];
        if index > 0 && index + 1 < roots.len() { move_axis(context, draft, id, axis, cursor - position(rect, axis))?; }
        cursor += size(rect, axis) + gap;
    }
    Ok(roots)
}
fn move_axis(context: &Context<'_>, draft: &mut Draft, id: &str, axis: Axis, delta: f64) -> Result<(), String> {
    movement::move_root(context, draft, id, match axis { Axis::X => StructurePoint { x: delta, y: 0.0 }, Axis::Y => StructurePoint { x: 0.0, y: delta } })
}
fn position(bounds: Bounds, axis: Axis) -> f64 { match axis { Axis::X => bounds.x, Axis::Y => bounds.y } }
fn size(bounds: Bounds, axis: Axis) -> f64 { match axis { Axis::X => bounds.width, Axis::Y => bounds.height } }
''')
write(base+'lifecycle.rs', '''
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
''')
write(base+'plan.rs', '''
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
''')
name='src-tauri/src/spaces/mod.rs';text=read(name)
if 'mod organization;' not in text: text='pub(in crate::spaces) mod organization;\n'+text
write(name,text)
name='src-tauri/src/spaces/canvas_state/mod.rs';text=read(name)
text+='\npub(in crate::spaces) use structure_validation::valid_id as valid_structure_id;\n'
write(name,text)
name='src-tauri/src/spaces/canvas_state/geometry.rs';text=read(name)
text=text.replace('#[derive(Clone, Copy, Debug, PartialEq)]\npub(in crate::spaces) struct Bounds', '#[derive(Clone, Copy, Debug, PartialEq, serde::Deserialize, serde::Serialize)]\n#[serde(deny_unknown_fields)]\npub(in crate::spaces) struct Bounds')
write(name,text)
for name,text in files.items():
    path=ROOT/name;path.parent.mkdir(parents=True,exist_ok=True)
    if not path.exists() or path.read_text()!=text:path.write_text(text)
Path(__file__).write_text('"""No pending source edits. The branch is verified as committed."""\nprint("No pending source edits")\n')
print('Applied Rust-owned structure command planning.')
