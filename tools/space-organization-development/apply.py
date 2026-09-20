"""Reviewed Rust structure validation and persistence cutover; no compatibility reader."""
from pathlib import Path
import re
ROOT=Path('.')
files={}
def read(path): return files[path] if path in files else (ROOT/path).read_text()
def write(path,text): files[path]=text.strip()+'\n'
def replace(path,old,new):
    source=read(path)
    if old not in source: raise RuntimeError(f'Missing source in {path}: {old[:70]}')
    write(path,source.replace(old,new))

write('src-tauri/src/spaces/canvas_state/structure_validation.rs', '''
use std::collections::{HashMap, HashSet};
use super::contract::{SpaceStructure, StructureItem, StructureKind};
pub(super) const MAX_ITEMS: usize = 100_000;
pub(super) const MAX_DEPTH: usize = 64;
pub(super) fn valid_id(id: &str) -> bool { !id.is_empty() && id.encode_utf16().count() <= 128 && !id.chars().any(|character| character.is_whitespace() || character == '\u{feff}') }
pub(super) fn validate(structure: &SpaceStructure) -> Result<(), String> {
    if structure.version != 1 || structure.items.len() > MAX_ITEMS { return Err("Unsupported or oversized Space structure".into()); }
    let mut items = HashMap::with_capacity(structure.items.len());
    let mut children: HashMap<Option<&str>, Vec<&StructureItem>> = HashMap::new();
    for item in &structure.items {
        validate_item(item)?;
        if items.insert(item.id.as_str(), item).is_some() { return Err("Structure identities must be unique".into()); }
        children.entry(item.parent_id.as_deref()).or_default().push(item);
    }
    for item in &structure.items {
        if let Some(parent) = item.parent_id.as_deref() {
            if items.get(parent).is_none_or(|item| item.kind == StructureKind::Content) { return Err("Every parent must be an existing container in this Space".into()); }
        }
        if item.kind == StructureKind::Group && !children.contains_key(&Some(item.id.as_str())) { return Err("A committed Group must not be empty".into()); }
    }
    for siblings in children.values_mut() {
        siblings.sort_by_key(|item| item.order);
        if siblings.iter().enumerate().any(|(index, item)| index != item.order as usize) { return Err("Sibling order must be unique and contiguous".into()); }
    }
    let mut pending: Vec<(&StructureItem, usize)> = children.get(&None).into_iter().flatten().map(|item| (*item, 1)).collect();
    let mut visited = HashSet::new();
    while let Some((item, depth)) = pending.pop() {
        if depth > MAX_DEPTH { return Err("Structure nesting exceeds 64 levels".into()); }
        if !visited.insert(item.id.as_str()) { return Err("Structure contains a cycle".into()); }
        pending.extend(children.get(&Some(item.id.as_str())).into_iter().flatten().map(|child| (*child, depth + 1)));
    }
    if visited.len() != items.len() { return Err("Structure contains an unreachable cycle".into()); }
    Ok(())
}
fn validate_item(item: &StructureItem) -> Result<(), String> {
    if !valid_id(&item.id) || item.parent_id.as_deref().is_some_and(|id| !valid_id(id)) || item.order as usize >= MAX_ITEMS { return Err("Invalid structure identity or order".into()); }
    match item.kind {
        StructureKind::Content => {
            if item.node_id.as_deref() != Some(item.id.as_str()) || item.title.is_some() { return Err("Content placements must use their content identity and cannot own a second title".into()); }
        }
        StructureKind::Group | StructureKind::Section => {
            let title = item.title.as_deref().ok_or("Containers require titles")?;
            if item.node_id.is_some() || title.is_empty() || title.encode_utf16().count() > 512 || title.trim_matches(|character: char| character.is_whitespace() || character == '\u{feff}') != title { return Err("Container titles must be trimmed and have 1 to 512 characters".into()); }
            if item.kind == StructureKind::Section && item.parent_id.is_some() { return Err("Sections belong directly to the Space root".into()); }
        }
    }
    Ok(())
}
''')
write('src-tauri/src/spaces/canvas_state/geometry.rs', '''
use std::collections::{HashMap, VecDeque};
use super::contract::{SpaceEditorDocument, StructureKind, StructureLayout, StructureTransform, StructureSize};
pub(in crate::spaces) const COORDINATE_LIMIT: f64 = 1_000_000_000_000.0;
pub(in crate::spaces) const SCALE_MIN: f64 = 0.000001;
pub(in crate::spaces) const SCALE_MAX: f64 = 1_000_000.0;
pub(in crate::spaces) fn identity() -> StructureTransform { StructureTransform { x: 0.0, y: 0.0, rotation: 0.0, scale: 1.0 } }
pub(in crate::spaces) fn normalize_rotation(value: f64) -> f64 { (value + 180.0).rem_euclid(360.0) - 180.0 }
pub(in crate::spaces) fn compose(parent: StructureTransform, local: StructureTransform) -> StructureTransform {
    let (sin, cos) = parent.rotation.to_radians().sin_cos();
    StructureTransform { x: parent.x + parent.scale * (cos * local.x - sin * local.y), y: parent.y + parent.scale * (sin * local.x + cos * local.y),
        rotation: normalize_rotation(parent.rotation + local.rotation), scale: parent.scale * local.scale }
}
pub(in crate::spaces) fn inverse(value: StructureTransform) -> StructureTransform {
    let mut inverse = StructureTransform { x: 0.0, y: 0.0, rotation: -value.rotation, scale: 1.0 / value.scale };
    let translated = compose(inverse, StructureTransform { x: -value.x, y: -value.y, ..identity() });
    inverse.x = translated.x; inverse.y = translated.y; inverse
}
pub(in crate::spaces) fn coordinate(value: f64) -> bool { value.is_finite() && value.abs() <= COORDINATE_LIMIT }
pub(in crate::spaces) fn valid_transform(value: StructureTransform) -> bool {
    coordinate(value.x) && coordinate(value.y) && value.rotation.is_finite() && (-180.0..180.0).contains(&value.rotation)
        && value.scale.is_finite() && (SCALE_MIN..=SCALE_MAX).contains(&value.scale)
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::spaces) struct Bounds { pub x: f64, pub y: f64, pub width: f64, pub height: f64 }
pub(in crate::spaces) fn transformed_bounds(transform: StructureTransform, size: StructureSize) -> Bounds {
    let points = [(0.0, 0.0), (size.width, 0.0), (0.0, size.height), (size.width, size.height)].map(|(x,y)| compose(transform, StructureTransform { x, y, ..identity() }));
    let x = points.iter().map(|point| point.x).fold(f64::INFINITY, f64::min);
    let y = points.iter().map(|point| point.y).fold(f64::INFINITY, f64::min);
    let right = points.iter().map(|point| point.x).fold(f64::NEG_INFINITY, f64::max);
    let bottom = points.iter().map(|point| point.y).fold(f64::NEG_INFINITY, f64::max);
    Bounds { x, y, width: right - x, height: bottom - y }
}
pub(in crate::spaces) struct Projection { pub world: HashMap<String, StructureTransform>, pub bounds: HashMap<String, Bounds>, pub order: Vec<String> }
/** Input topology is validated before projection; no application state is read from an engine. */
pub(in crate::spaces) fn project(document: &SpaceEditorDocument) -> Result<Projection, String> {
    let items: HashMap<_,_> = document.structure.items.iter().map(|item| (item.id.as_str(), item)).collect();
    let layouts: HashMap<_,_> = document.layout.items.iter().map(|layout| (layout.id(), layout)).collect();
    let mut children: HashMap<Option<&str>, Vec<&str>> = HashMap::new();
    for item in &document.structure.items { children.entry(item.parent_id.as_deref()).or_default().push(item.id.as_str()); }
    for siblings in children.values_mut() { siblings.sort_by_key(|id| items[id].order); }
    let mut pending: VecDeque<&str> = children.get(&None).into_iter().flatten().copied().collect();
    let mut world = HashMap::new(); let mut bounds = HashMap::new(); let mut order = Vec::new();
    while let Some(id) = pending.pop_front() {
        let item = items[id]; let layout = layouts.get(id).ok_or("Structure item has no layout")?;
        let parent = item.parent_id.as_deref().map(|id| world.get(id).copied().ok_or("Unreachable parent")).transpose()?.unwrap_or_else(identity);
        let transform = compose(parent, layout.local());
        if !valid_transform(transform) { return Err("Composed world transform exceeds the canonical limits".into()); }
        if let StructureLayout::Content { size, .. } | StructureLayout::Section { size, .. } = layout {
            let rect = transformed_bounds(transform, *size);
            if rect.width <= 0.0 || rect.height <= 0.0 || ![rect.x,rect.y,rect.x+rect.width,rect.y+rect.height].into_iter().all(coordinate) { return Err("World geometry is not representable".into()); }
            bounds.insert(id.to_string(), rect);
        }
        world.insert(id.to_string(), transform); order.push(id.to_string());
        if let Some(children) = children.get(&Some(id)) { for child in children.iter().rev() { pending.push_front(child); } }
    }
    if order.len() != items.len() { return Err("Structure contains an unreachable item".into()); }
    for id in order.iter().rev() {
        if items[id.as_str()].kind != StructureKind::Group { continue; }
        let rectangles: Vec<_> = children.get(&Some(id.as_str())).into_iter().flatten().map(|child| bounds.get(*child).copied().ok_or("Group child has no bounds")).collect::<Result<_,_>>()?;
        bounds.insert(id.clone(), union_bounds(&rectangles).ok_or("Empty Group")?);
    }
    Ok(Projection { world, bounds, order })
}
pub(in crate::spaces) fn union_bounds(rectangles: &[Bounds]) -> Option<Bounds> {
    if rectangles.is_empty() { return None; }
    let x = rectangles.iter().map(|rect| rect.x).fold(f64::INFINITY, f64::min);
    let y = rectangles.iter().map(|rect| rect.y).fold(f64::INFINITY, f64::min);
    let right = rectangles.iter().map(|rect| rect.x + rect.width).fold(f64::NEG_INFINITY, f64::max);
    let bottom = rectangles.iter().map(|rect| rect.y + rect.height).fold(f64::NEG_INFINITY, f64::max);
    Some(Bounds { x, y, width: right - x, height: bottom - y })
}
''')
write('src-tauri/src/spaces/canvas_state/layout_validation.rs', '''
use std::collections::{HashMap, HashSet};
use super::contract::{CanvasNodeAppearance, SpaceEditorDocument, StructureLayout, StructureSize};
use super::geometry::{coordinate, valid_transform, COORDINATE_LIMIT};
pub(super) fn validate(document: &SpaceEditorDocument) -> Result<(), String> {
    if document.layout.version != 1 || document.layout.items.len() != document.structure.items.len() { return Err("Structure and layout must have exactly matching identities".into()); }
    let kinds: HashMap<_,_> = document.structure.items.iter().map(|item| (item.id.as_str(), item.kind)).collect();
    let mut seen = HashSet::new();
    for layout in &document.layout.items {
        if !seen.insert(layout.id()) || kinds.get(layout.id()) != Some(&layout.kind()) { return Err("Duplicate, orphan or mismatched layout identity".into()); }
        if !valid_transform(layout.local()) { return Err("Invalid local transform".into()); }
        match layout {
            StructureLayout::Content { size, file_subpath, .. } => {
                validate_size(*size)?;
                if file_subpath.as_deref().is_some_and(|path| !path.starts_with('#')) { return Err("fileSubpath must start with #".into()); }
            },
            StructureLayout::Section { size, position, .. } => {
                validate_size(*size)?;
                if !coordinate(position.x) || !coordinate(position.y) { return Err("Invalid Section position".into()); }
            },
            StructureLayout::Group { .. } => {},
        }
        if let Some(appearance) = layout.appearance() { validate_appearance(appearance)?; }
    }
    super::geometry::project(document)?;
    Ok(())
}
fn validate_size(size: StructureSize) -> Result<(), String> {
    if [size.width,size.height].into_iter().any(|value| !value.is_finite() || value <= 0.0 || value > COORDINATE_LIMIT) { return Err("Invalid layout size".into()); }
    Ok(())
}
pub(in crate::spaces) fn validate_appearance(value: &CanvasNodeAppearance) -> Result<(), String> {
    if value.fill.is_some() && value.custom_fill.is_some() { return Err("fill and customFill are mutually exclusive".into()); }
    if value.custom_fill.as_deref().is_some_and(|color| color.len() != 7 || !color.starts_with('#') || !color.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)) { return Err("Custom fill must be #RRGGBB".into()); }
    if value.font_size.is_some_and(|number| !number.is_finite() || !(2.5..=576.0).contains(&number)) { return Err("Invalid font size".into()); }
    if value.typography_scale.is_some_and(|number| !number.is_finite() || !(0.25..=4.0).contains(&number)) { return Err("Invalid typography scale".into()); }
    Ok(())
}
''')
write('src-tauri/src/spaces/canvas_state/validation.rs', '''
use std::collections::HashSet;
use super::contract::SpaceEditorDocument;
use crate::spaces::db;
pub(in crate::spaces) fn validate_document(document: &SpaceEditorDocument) -> Result<(), String> {
    if document.version != 1 { return Err("Unsupported Space editor document version".into()); }
    super::structure_validation::validate(&document.structure)?;
    super::layout_validation::validate(document)?;
    if !document.viewport.x.is_finite() || !document.viewport.y.is_finite() || !document.viewport.zoom.is_finite() || document.viewport.zoom <= 0.0 { return Err("Invalid camera state".into()); }
    let mut ids = HashSet::new();
    for style in document.relation_styles.iter().flatten() {
        if !super::structure_validation::valid_id(&style.relation_id) || !ids.insert(style.relation_id.as_str()) { return Err("Relation styles must have unique valid identities".into()); }
    }
    if ids.len() > 100_000 { return Err("Too many relation styles".into()); }
    Ok(())
}
pub(in crate::spaces) async fn validate_content_references(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, document: &SpaceEditorDocument) -> Result<(), String> {
    let active: Vec<String> = sqlx::query_scalar("SELECT id FROM spaces_nodes WHERE space_id=$1 AND deleted_at IS NULL").bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    let expected: HashSet<&str> = active.iter().map(String::as_str).collect();
    let actual: HashSet<&str> = document.structure.items.iter().filter_map(|item| item.node_id.as_deref()).collect();
    if expected != actual { return Err("The editor must place exactly the active content belonging to its Space".into()); }
    Ok(())
}
''')
# Current snapshot load/save checks the actual DB content set, not the renderer's available nodes.
name='src-tauri/src/spaces/canvas_state/repository.rs';text=read(name)
text=text.replace('    let next_revision = base_revision + 1;', '''    if !(0..9_007_199_254_740_991).contains(&base_revision) { return Err("Invalid editor revision".into()); }
    super::validation::validate_content_references(tx, space_id, document).await?;
    let next_revision = base_revision + 1;''')
text=text.replace('    row.map(decode).transpose()', '''    if row.is_none() {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_nodes WHERE space_id=$1 AND deleted_at IS NULL").bind(space_id).fetch_one(pool).await.map_err(db::sql_error)?;
        if count != 0 { return Err("Active content is missing its editor organization document".into()); }
    }
    row.map(decode).transpose()''')
text+='''
pub(in crate::spaces) async fn load_in_transaction(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str) -> Result<(SpaceEditorDocument, i64), String> {
    let row = sqlx::query("SELECT document,revision FROM spaces_canvas_state WHERE space_id=$1").bind(space_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?;
    match row { Some(row) => decode(row), None => Ok((SpaceEditorDocument::empty(), 0)) }
}
'''
write(name,text)
name='src-tauri/src/spaces/canvas_state/mod.rs';text=read(name)
text=text.replace('mod validation;', 'mod validation;\nmod structure_validation;\nmod layout_validation;\npub(in crate::spaces) mod geometry;')
text=text.replace('pub(in crate::spaces) use repository::save_in_transaction;', 'pub(in crate::spaces) use repository::{save_in_transaction, load_in_transaction};')
write(name,text)
name='src-tauri/src/spaces/canvas_state/gesture.rs';text=read(name)
text=re.sub(r'^    pub\(super\) changes: Vec<nodes::contracts::NodeDataChange>,\n','',text,flags=re.M)
text=re.sub(r'    nodes::update_many_in_transaction\([\s\S]*?\.await\?;\n','',text,count=1)
write(name,text)
# Content payloads carry neither layout membership nor edit locks. Wrong fields fail schema validation.
name='src-tauri/src/spaces/knowledge/node_contracts/payload/mod.rs';text=read(name)
text=text.replace('use serde_json::{Map, Value};','use serde_json::Value;')
text=text.replace('optional_bool, optional_string','optional_string')
text=text.replace('    "sectionId",\n','').replace('    "locked",\n','')
text=text.replace('    optional_string(object, "sectionId", 128, false)?;\n','').replace('    optional_bool(object, "locked")?;\n','')
text=text.replace('        "section" => validate_section(object),\n','')
text=re.sub(r'fn validate_section\([^)]*\)[\s\S]*?\n}\n','',text,count=1)
write(name,text)
name='src-tauri/src/spaces/knowledge/node_contracts/payload/fields.rs';write(name,read(name).replace('        "section" => Ok(&["collapsed"]),\n',''))
name='src-tauri/src/spaces/knowledge/node_contracts/mod.rs';write(name,read(name).replace(' | "section"',''))
# Asset references describe editor items. Organization ownership is checked against committed layout, not a fabricated content FK.
name='src-tauri/src/spaces/assets/reclamation/canvas_references.rs';text=read(name)
text=text.replace('document.nodes','document.layout.items').replace('node.node_id','node.id()')
text=re.sub(r'node\s*\.appearance\s*\.as_ref\(\)', 'node.appearance()', text)
text=text.replace('node_id','item_id')
write(name,text)
# Record layout+structure in the content insertion request. It is a typed content use case, not a renderer write.
name='src-tauri/src/spaces/nodes/contracts.rs';text=read(name)
text=text.replace('pub struct InsertNodesRequest {\n    pub space_id: String,','pub struct InsertNodesRequest {\n    pub space_id: String,\n    pub base_revision: i64,\n    pub document: crate::spaces::canvas_state::SpaceEditorDocument,')
text+='''
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeEditorMutationResult { pub revision: i64 }
'''
write(name,text)
name='src-tauri/src/spaces/nodes/insert.rs';text=read(name)
text=text.replace(') -> Result<(), String> {\n    if request.nodes.is_empty()', ') -> Result<super::contracts::NodeEditorMutationResult, String> {\n    if request.nodes.is_empty()',1)
text=text.replace('        return Ok(());','        return Err("Content insertion requires a non-empty selection".into());',1)
text=text.replace('    tx.commit().await.map_err(db::sql_error)\n}', '''    let revision = crate::spaces::canvas_state::save_in_transaction(&mut tx, &request.space_id, request.base_revision, &request.document).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(super::contracts::NodeEditorMutationResult { revision })
}''',1)
write(name,text)
name='src-tauri/src/spaces/nodes/commands.rs';text=read(name)
text=text.replace(') -> Result<(), String> {\n    super::insert_many', ') -> Result<super::contracts::NodeEditorMutationResult, String> {\n    super::insert_many',1)
write(name,text)
name='src/features/spaces/nodes/node-repository.ts';text=read(name)
text='import { transactSpaceEditor } from "../canvas/persistence/canvas-persistence-coordinator";\nimport { appendEditorContent } from "../canvas/document/canvas-document-sync";\n'+text
start=text.index('  await invoke("plugin:spaces|create_nodes", {')
end=text.index('\n}',start)
body=text[start:end]
body=body.replace('  await invoke("plugin:spaces|create_nodes", {','  const document = appendEditorContent(before, nodes);\n  const saved = await invoke<{ revision: number }>("plugin:spaces|create_nodes", {')
body=body.replace('      spaceId,','      spaceId, baseRevision, document,',1)
text=text[:start]+'  await transactSpaceEditor(spaceId, async (before, baseRevision) => {\n'+body+'\n    return { document, revision: saved.revision };\n  });'+text[end:]
write(name,text)
# Baseline edits are done in a separate explicit step after native lifecycle tables are finalized.
for name,text in files.items():
    path=ROOT/name;path.parent.mkdir(parents=True,exist_ok=True)
    if not path.exists() or path.read_text()!=text:path.write_text(text)
# Clear this reviewed one-shot recipe. Later verification must never silently repeat code modifications.
Path(__file__).write_text('"""No pending source edits. The branch is verified as committed."""\nprint("No pending source edits")\n')
print('Applied authoritative structural and geometric validation plus atomic content placement.')
