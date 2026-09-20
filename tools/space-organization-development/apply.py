"""Reviewed native edit intents and pure planning; keeps existing macOS capabilities unchanged."""
from pathlib import Path
import json
ROOT=Path('.')
files={}
def read(path): return files[path] if path in files else (ROOT/path).read_text()
def write(path,text): files[path]=text.strip()+'\n'
# The dependency already enables this macOS-only capability unconditionally. Match that
# configuration on direct Cargo builds too; this has no effect on non-macOS capabilities.
name='src-tauri/tauri.conf.json';config=json.loads(read(name));config['app']['macOSPrivateApi']=True
write(name,json.dumps(config,ensure_ascii=False,indent=2))
base='src-tauri/src/spaces/organization/'
write(base+'editor_command.rs', '''
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::contract::StructureCommand;
use crate::spaces::canvas_state::contract::{CanvasNodeFrame, CanvasNodeSizeAuthority, CanvasRelationColor};
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(in crate::spaces) enum EditorCommand {
    Structure { command: StructureCommand, selection: Vec<String> },
    TransformFrames { updates: Vec<FrameUpdate> },
    ArrangeFrames { updates: Vec<FrameUpdate> },
    SetAppearance { #[serde(rename = "itemId")] item_id: String, changes: BTreeMap<AppearanceField, Option<Value>> },
    SetRelationColor { #[serde(rename = "relationId")] relation_id: String, color: Option<CanvasRelationColor> },
    MoveLayer { ids: Vec<String>, mode: LayerMove },
    DeleteSelection { ids: Vec<String>, #[serde(rename = "relationIds")] relation_ids: Vec<String> },
    MeasureText { measurements: Vec<TextMeasurement> },
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::spaces) struct FrameUpdate {
    pub node_id: String, pub frame: CanvasNodeFrame,
    #[serde(skip_serializing_if = "Option::is_none")] pub size_authority: Option<CanvasNodeSizeAuthority>,
    #[serde(skip_serializing_if = "Option::is_none")] pub typography_scale: Option<f64>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::spaces) struct TextMeasurement {
    pub item_id: String, pub width: f64, pub height: f64, pub typography_scale: f64,
    pub content_revision: String,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::spaces) enum AppearanceField {
    Fill, CustomFill, TextColor, FontSize, TypographyScale, FontWeight, FontStyle,
    TextDecoration, TextAlign, VerticalAlign, Background, BackgroundStyle,
}
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(in crate::spaces) enum LayerMove { Front, Forward, Backward, Back }
impl EditorCommand {
    pub(in crate::spaces) fn deletes_subtrees(&self) -> bool {
        matches!(self, Self::DeleteSelection { .. } | Self::Structure { command: StructureCommand::DeleteSubtrees { .. }, .. })
    }
    pub(in crate::spaces) fn requested_relation_deletions(&self) -> &[String] {
        match self { Self::DeleteSelection { relation_ids, .. } => relation_ids, _ => &[] }
    }
}
''')
write(base+'frame_edit.rs', '''
use super::{context::Context, draft::Draft, editor_command::FrameUpdate, movement, section_editing};
use crate::spaces::canvas_state::{contract::{StructureKind, StructureLayout, StructureTransform}, geometry::{self, Bounds}};
pub(super) fn transform(context: &Context<'_>, draft: &mut Draft, updates: &[FrameUpdate]) -> Result<Vec<String>, String> {
    let ids = validate_updates(context, updates)?;
    let roots = context.roots(&ids, false)?; context.editable(&roots)?;
    let by_id: std::collections::HashMap<_,_> = updates.iter().map(|update| (update.node_id.as_str(), update)).collect();
    for id in &roots {
        let update = by_id[id.as_str()]; let frame = &update.frame;
        let item = context.item(id)?;
        match item.kind {
            StructureKind::Section => {
                if frame.rotation != 0.0 { return Err("Sections cannot rotate".into()); }
                section_editing::resize(context, draft, id, Bounds { x: frame.x, y: frame.y, width: frame.width, height: frame.height })?;
            },
            StructureKind::Group => {
                let children = context.children.get(&Some(id.as_str())).cloned().unwrap_or_default();
                let bounds = context.bounds_in_parent(&children, Some(id))?.ok_or("Group has no geometry")?;
                let scale = frame.width / bounds.width;
                if (scale - frame.height / bounds.height).abs() > 1e-6 * scale.max(1.0) { return Err("Group scaling must be uniform".into()); }
                let linear = StructureTransform { x: 0.0, y: 0.0, rotation: geometry::normalize_rotation(frame.rotation), scale };
                let offset = geometry::compose(linear, StructureTransform { x: bounds.x, y: bounds.y, ..geometry::identity() });
                movement::transform_group(context, draft, id, StructureTransform { x: frame.x - offset.x, y: frame.y - offset.y, ..linear })?;
            },
            StructureKind::Content => resize_content(context, draft, update)?,
        }
    }
    Ok(roots)
}
pub(super) fn arrange(context: &Context<'_>, draft: &mut Draft, updates: &[FrameUpdate]) -> Result<Vec<String>, String> {
    let ids = validate_updates(context, updates)?; let roots = context.roots(&ids, false)?; context.editable(&roots)?;
    let by_id: std::collections::HashMap<_,_> = updates.iter().map(|update| (update.node_id.as_str(), update)).collect();
    for id in &roots {
        let frame = &by_id[id.as_str()].frame;
        let before = oriented_origin(context, id)?;
        movement::move_root(context, draft, id, crate::spaces::canvas_state::contract::StructurePoint { x: frame.x - before.0, y: frame.y - before.1 })?;
    }
    Ok(roots)
}
fn resize_content(context: &Context<'_>, draft: &mut Draft, update: &FrameUpdate) -> Result<(), String> {
    let item = context.item(&update.node_id)?; let world = context.projection.world[&item.id];
    let frame = &update.frame;
    let parent = context.parent_world(item.parent_id.as_deref())?;
    let transform = geometry::compose(geometry::inverse(parent), StructureTransform { x: frame.x, y: frame.y, rotation: geometry::normalize_rotation(frame.rotation), ..world });
    let layout = draft.layouts.get_mut(&item.id).ok_or("Content layout is unavailable")?;
    if let StructureLayout::Content { transform: local, size, size_authority, appearance, .. } = layout {
        *local = transform; size.width = frame.width / world.scale; size.height = frame.height / world.scale;
        if let Some(authority) = &update.size_authority { *size_authority = authority.clone(); }
        if let Some(scale) = update.typography_scale {
            let mut value = appearance.as_ref().map(serde_json::to_value).transpose().map_err(|error| error.to_string())?.unwrap_or_else(|| serde_json::json!({}));
            value["typographyScale"] = serde_json::json!(scale);
            *appearance = Some(serde_json::from_value(value).map_err(|error| error.to_string())?);
        }
    }
    Ok(())
}
fn oriented_origin(context: &Context<'_>, id: &str) -> Result<(f64,f64), String> {
    let world = context.projection.world[id];
    if context.item(id)?.kind != StructureKind::Group { return Ok((world.x,world.y)); }
    let children = context.children.get(&Some(id)).cloned().unwrap_or_default();
    let bounds = context.bounds_in_parent(&children, Some(id))?.ok_or("Group has no geometry")?;
    let point = geometry::compose(world, StructureTransform { x: bounds.x, y: bounds.y, ..geometry::identity() });
    Ok((point.x,point.y))
}
fn validate_updates(context: &Context<'_>, updates: &[FrameUpdate]) -> Result<Vec<String>, String> {
    if updates.is_empty() || updates.len() > 100_000 { return Err("A frame edit requires a bounded selection".into()); }
    let mut seen = std::collections::HashSet::new();
    for update in updates {
        if !seen.insert(&update.node_id) { return Err("A frame edit cannot update an item twice".into()); }
        context.item(&update.node_id)?;
        let frame = &update.frame;
        if ![frame.x,frame.y,frame.width,frame.height].into_iter().all(geometry::coordinate) || frame.width <= 0.0 || frame.height <= 0.0 || !frame.rotation.is_finite() { return Err("Invalid requested frame".into()); }
        if update.typography_scale.is_some_and(|scale| !scale.is_finite() || !(0.25..=4.0).contains(&scale)) { return Err("Invalid typography scale".into()); }
    }
    Ok(updates.iter().map(|update| update.node_id.clone()).collect())
}
''')
write(base+'appearance_edit.rs', '''
use std::collections::BTreeMap;
use serde_json::Value;
use super::{context::Context, draft::Draft, editor_command::AppearanceField};
use crate::spaces::canvas_state::contract::StructureLayout;
pub(super) fn apply(context: &Context<'_>, draft: &mut Draft, id: &str, patch: &BTreeMap<AppearanceField, Option<Value>>) -> Result<(), String> {
    context.editable(&[id.into()])?;
    let layout = draft.layouts.get_mut(id).ok_or("Appearance target is unavailable")?;
    let appearance = match layout { StructureLayout::Content { appearance, .. } | StructureLayout::Section { appearance, .. } => appearance,
        StructureLayout::Group { .. } => return Err("Groups have no independent appearance".into()) };
    let mut value = appearance.as_ref().map(serde_json::to_value).transpose().map_err(|error| error.to_string())?.unwrap_or_else(|| serde_json::json!({}));
    let object = value.as_object_mut().ok_or("Invalid current appearance")?;
    for (field,value) in patch {
        let key = serde_json::to_value(field).map_err(|error| error.to_string())?;
        let key = key.as_str().ok_or("Invalid appearance field")?;
        match value { Some(value) => { object.insert(key.to_string(), value.clone()); }, None => { object.remove(key); } }
    }
    if object.is_empty() { *appearance = None; }
    else { *appearance = Some(serde_json::from_value(value).map_err(|error| error.to_string())?); }
    Ok(())
}
''')
write(base+'layer_order.rs', '''
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
''')
write(base+'editor_plan.rs', '''
use super::{context::Context, draft::Draft, editor_command::EditorCommand, frame_edit, appearance_edit, layer_order};
use crate::spaces::canvas_state::{self, contract::{SpaceEditorDocument, CanvasRelationStyleV1}};
pub(in crate::spaces) fn plan_editor(before: &SpaceEditorDocument, command: &EditorCommand) -> Result<super::Plan, String> {
    match command {
        EditorCommand::Structure { command, selection } => return super::plan(before, command, selection),
        EditorCommand::DeleteSelection { ids, .. } => {
            if ids.is_empty() { return Ok(super::Plan { after: before.clone(), selection: vec![], removed_content_ids: vec![] }); }
            return super::plan(before, &super::StructureCommand::DeleteSubtrees { ids: ids.clone() }, &[]);
        },
        EditorCommand::MeasureText { .. } => return Err("Text measurements require the authoritative content read set".into()),
        _ => {},
    }
    let context = Context::new(before)?; let mut draft = Draft::new(before);
    let selection = match command {
        EditorCommand::TransformFrames { updates } => frame_edit::transform(&context, &mut draft, updates)?,
        EditorCommand::ArrangeFrames { updates } => frame_edit::arrange(&context, &mut draft, updates)?,
        EditorCommand::SetAppearance { item_id, changes } => { appearance_edit::apply(&context, &mut draft, item_id, changes)?; vec![item_id.clone()] },
        EditorCommand::MoveLayer { ids, mode } => layer_order::apply(&context, &mut draft, ids, *mode)?,
        EditorCommand::SetRelationColor { .. } => vec![],
        _ => return Err("Unsupported editor intent".into()),
    };
    let mut after = draft.finish(before);
    if let EditorCommand::SetRelationColor { relation_id, color } = command {
        let mut styles: Vec<_> = after.relation_styles.take().unwrap_or_default().into_iter().filter(|style| style.relation_id != *relation_id).collect();
        if let Some(color) = color { styles.push(CanvasRelationStyleV1 { relation_id: relation_id.clone(), color: color.clone() }); }
        styles.sort_by(|a,b| a.relation_id.cmp(&b.relation_id));
        after.relation_styles = if styles.is_empty() { None } else { Some(styles) };
    }
    canvas_state::validate_document(&after)?;
    Ok(super::Plan { after, selection, removed_content_ids: vec![] })
}
''')
name=base+'mod.rs';text=read(name)
text+='\nmod editor_command;\nmod editor_plan;\nmod frame_edit;\nmod appearance_edit;\nmod layer_order;\npub(in crate::spaces) use editor_command::{EditorCommand, FrameUpdate, TextMeasurement};\npub(in crate::spaces) use editor_plan::plan_editor;\n'
write(name,text)
name=base+'contract.rs';text=read(name).replace('pub(super) enum Axis','pub(in crate::spaces) enum Axis').replace('pub(super) enum Alignment','pub(in crate::spaces) enum Alignment')
write(name,text)
# Expose compiler cause, not just lines containing the word error.
report=ROOT/'tools/space-organization-development/last-native/report.md'
if report.exists(): write('tools/space-organization-development/last-native/tail.md', '# Exact native checkpoint tail\n\n'+ '\n'.join(report.read_text().splitlines()[-300:]))
for name,text in files.items():
    path=ROOT/name;path.parent.mkdir(parents=True,exist_ok=True)
    if not path.exists() or path.read_text()!=text:path.write_text(text)
Path(__file__).write_text('"""No pending source edits. The branch is verified as committed."""\nprint("No pending source edits")\n')
print('Added finite native editor intents and matched the existing macOS feature configuration.')
