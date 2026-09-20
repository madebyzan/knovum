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
