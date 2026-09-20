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
