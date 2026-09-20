"""Integrate native fragment insertion with revisioned commands and reversible receipts."""
from pathlib import Path

root = Path.cwd()
marker = root / "tools/space-organization-development/applied/fragment-execution"
if marker.exists():
    raise SystemExit(0)
sources = {}


def read(path):
    return sources[path] if path in sources else (root / path).read_text()


def write(path, text):
    sources[path] = text.strip() + "\n"


def replace(path, before, after):
    source = read(path)
    if before not in source:
        raise RuntimeError(f"Reviewed source precondition failed: {path}: {before[:100]}")
    write(path, source.replace(before, after))


write("src-tauri/src/spaces/organization/fragment_contract.rs", r'''
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::spaces::{canvas_state::contract::{SpaceStructure, StructureLayoutDocument, StructurePoint, CanvasRelationStyleV1}, knowledge::indexing::KnowledgeChunkInput};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::spaces) struct EditorFragment {
    pub structure: SpaceStructure,
    pub layout: StructureLayoutDocument,
    pub nodes: Vec<FragmentNode>,
    pub relations: Vec<FragmentRelation>,
    pub object_types: Vec<FragmentObjectType>,
    pub relation_styles: Vec<CanvasRelationStyleV1>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::spaces) struct FragmentNode {
    pub id: String, pub kind: String, pub data: Value, pub source_chunks: Vec<KnowledgeChunkInput>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::spaces) struct FragmentRelation {
    pub id: String, pub source_id: String, pub target_id: String, pub kind: String,
    pub rationale: String, pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")] pub confidence: Option<f64>,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::spaces) struct FragmentObjectType {
    pub id: String, pub name: String, pub icon: String, pub description: String, pub fields: Value,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::spaces) struct FragmentPlacement { pub parent_id: Option<String>, pub delta: StructurePoint }
''')
replace("src-tauri/src/spaces/knowledge/indexing/mod.rs", "#[derive(Debug, Deserialize)]\n#[serde(rename_all = \"camelCase\")]\npub struct KnowledgeChunkInput", "#[derive(Clone, Debug, Deserialize, Serialize)]\n#[serde(rename_all = \"camelCase\", deny_unknown_fields)]\npub struct KnowledgeChunkInput")
replace("src-tauri/src/spaces/organization/mod.rs", "mod editor_command;", "mod editor_command;\nmod fragment_contract;\nmod fragment_plan;\npub(in crate::spaces) use fragment_contract::{EditorFragment, FragmentPlacement, FragmentObjectType};")
replace("src-tauri/src/spaces/organization/editor_command.rs", "    MeasureText { measurements: Vec<TextMeasurement> },", "    MeasureText { measurements: Vec<TextMeasurement> },\n    InsertFragment { fragment: super::fragment_contract::EditorFragment, placement: super::fragment_contract::FragmentPlacement },")
replace("src-tauri/src/spaces/organization/editor_command.rs", "impl EditorCommand {", "impl EditorCommand {\n    pub(in crate::spaces) fn changes_content(&self) -> bool { self.deletes_subtrees() || matches!(self, Self::InsertFragment { .. }) }")
replace("src-tauri/src/spaces/organization/editor_plan.rs", "    match command {", "    match command {\n        EditorCommand::InsertFragment { fragment, placement } => return super::fragment_plan::plan(before, fragment, placement),")
write("src-tauri/src/spaces/organization/fragment_plan.rs", r'''
use std::collections::HashSet;
use crate::spaces::canvas_state::{self, contract::{SpaceEditorDocument, StructureKind, StructureLayout}, geometry};
use super::{context::Context, fragment_contract::{EditorFragment, FragmentPlacement}, Plan};

pub(super) fn plan(before: &SpaceEditorDocument, fragment: &EditorFragment, placement: &FragmentPlacement) -> Result<Plan, String> {
    validate_fragment(fragment)?;
    let context = Context::new(before)?;
    if let Some(parent) = &placement.parent_id {
        context.editable(std::slice::from_ref(parent))?;
        if context.item(parent)?.kind == StructureKind::Content { return Err("Content cannot contain pasted items".into()); }
    }
    if fragment.structure.items.iter().any(|item| context.items.contains_key(item.id.as_str())) { return Err("Inserted structure identities already exist in the target Space".into()); }
    if !placement.delta.x.is_finite() || !placement.delta.y.is_finite() { return Err("Insertion offset must be finite".into()); }
    let inverse = geometry::inverse(context.parent_world(placement.parent_id.as_deref())?);
    let mut after = before.clone();
    let first = context.children.get(&placement.parent_id.as_deref()).map_or(0, Vec::len) as u32;
    let roots: HashSet<&str> = fragment.structure.items.iter().filter(|item| item.parent_id.is_none()).map(|item| item.id.as_str()).collect();
    let mut selection = Vec::new();
    let mut items = fragment.structure.items.clone();
    items.sort_by_key(|item| item.order);
    for mut item in items {
        if roots.contains(item.id.as_str()) {
            if item.kind == StructureKind::Section && placement.parent_id.is_some() { return Err("Sections can only be inserted at the Space root".into()); }
            selection.push(item.id.clone()); item.parent_id = placement.parent_id.clone(); item.order += first;
        }
        after.structure.items.push(item);
    }
    for mut layout in fragment.layout.items.clone() {
        if roots.contains(layout.id()) {
            match &mut layout {
                StructureLayout::Content { transform, .. } | StructureLayout::Group { transform, .. } => {
                    transform.x += placement.delta.x; transform.y += placement.delta.y;
                    *transform = geometry::compose(inverse, *transform);
                }
                StructureLayout::Section { position, .. } => { position.x += placement.delta.x; position.y += placement.delta.y; }
            }
        }
        after.layout.items.push(layout);
    }
    if !fragment.relation_styles.is_empty() { after.relation_styles.get_or_insert_with(Vec::new).extend(fragment.relation_styles.iter().cloned()); }
    canvas_state::validate_document(&after)?;
    Ok(Plan { after, selection, removed_content_ids: vec![] })
}
fn validate_fragment(fragment: &EditorFragment) -> Result<(), String> {
    if fragment.structure.items.is_empty() || fragment.structure.items.len() > 10_000 || fragment.nodes.len() > 10_000 || fragment.relations.len() > 100_000 || fragment.object_types.len() > 10_000 { return Err("A native fragment must contain 1 to 10,000 structural items".into()); }
    let mut document = SpaceEditorDocument::empty(); document.structure = fragment.structure.clone(); document.layout = fragment.layout.clone(); document.relation_styles = Some(fragment.relation_styles.clone());
    canvas_state::validate_document(&document)?;
    let placed: HashSet<&str> = fragment.structure.items.iter().filter_map(|item| item.node_id.as_deref()).collect();
    let nodes: HashSet<&str> = fragment.nodes.iter().map(|node| node.id.as_str()).collect();
    if placed != nodes || nodes.len() != fragment.nodes.len() { return Err("Fragment content must match its structure exactly".into()); }
    let mut relations = HashSet::new();
    for relation in &fragment.relations {
        if !canvas_state::valid_structure_id(&relation.id) || !relations.insert(relation.id.as_str()) || !nodes.contains(relation.source_id.as_str()) || !nodes.contains(relation.target_id.as_str()) { return Err("Fragment relationships require unique identities and internal endpoints".into()); }
        if !matches!(relation.status.as_str(), "active" | "contested") { return Err("Only current relationships may be inserted".into()); }
    }
    if fragment.relation_styles.iter().any(|style| !relations.contains(style.relation_id.as_str())) { return Err("Fragment relationship appearance must have an internal relationship".into()); }
    let mut types = HashSet::new();
    for definition in &fragment.object_types {
        if !types.insert(definition.id.as_str()) || !fragment.nodes.iter().any(|node| node.data.get("objectTypeId").and_then(serde_json::Value::as_str) == Some(definition.id.as_str())) { return Err("Inserted custom types must be unique and used by the inserted content".into()); }
    }
    Ok(())
}
''')
write("src-tauri/src/spaces/editing/fragment_types.rs", r'''
use crate::spaces::{db, knowledge::{events, types}, organization::FragmentObjectType};

pub(super) async fn insert(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, definitions: &[FragmentObjectType], now: i64) -> Result<(), String> {
    for definition in definitions {
        types::validate_custom_type_definition(&definition.id, &definition.name, &definition.icon, &definition.description, &definition.fields)?;
        sqlx::query("INSERT INTO spaces_object_types(id,space_id,name,icon,description,fields,system,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,0,$7,$7)")
            .bind(&definition.id).bind(space_id).bind(&definition.name).bind(&definition.icon).bind(&definition.description)
            .bind(serde_json::to_string(&definition.fields).map_err(db::json_error)?).bind(now).execute(&mut **tx).await.map_err(db::sql_error)?;
        events::append(tx, events::EventRecord::new(space_id, events::EventEntity::new("type", &definition.id), "created", events::EventActor::new("human", None), &serde_json::json!({"name": definition.name}), now)).await?;
    }
    Ok(())
}
''')
write("src-tauri/src/spaces/editing/fragment_execution.rs", r'''
use crate::spaces::{db, canvas_state::{self, SpaceEditorDocument}, nodes::{self, contracts::InsertNode}, organization::EditorFragment, commands::relations::{self, SpaceRelationInput}};

pub(super) async fn stage(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, fragment: &EditorFragment, after: &SpaceEditorDocument, now: i64) -> Result<Vec<InsertNode>, String> {
    let ids: Vec<_> = fragment.structure.items.iter().map(|item| &item.id).collect();
    let archived: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_editor_trash WHERE space_id=$1 AND item_id IN (SELECT value FROM json_each($2))")
        .bind(space_id).bind(serde_json::to_string(&ids).map_err(db::json_error)?).fetch_one(&mut **tx).await.map_err(db::sql_error)?;
    if archived != 0 { return Err("Inserted identities are still owned by Recently Deleted".into()); }
    super::fragment_types::insert(tx, space_id, &fragment.object_types, now).await?;
    let projection = canvas_state::geometry::project(after)?;
    let content: Vec<_> = fragment.nodes.iter().map(|node| {
        let world = projection.world[&node.id];
        InsertNode { id: node.id.clone(), kind: node.kind.clone(), initial_x: world.x, initial_y: world.y, data: node.data.clone(), source_chunks: node.source_chunks.clone() }
    }).collect();
    nodes::stage_nodes_in_transaction(tx, space_id, &content, now).await?;
    for relation in &fragment.relations {
        relations::create_relation_in_transaction(tx, &SpaceRelationInput {
            id: relation.id.clone(), space_id: space_id.to_string(), source_id: relation.source_id.clone(), target_id: relation.target_id.clone(),
            kind: relation.kind.clone(), rationale: relation.rationale.clone(), status: relation.status.clone(), confidence: relation.confidence,
        }, "human", None, now).await?;
    }
    Ok(content)
}
''')
# Stage every row before validating references. No foreign-key disabling or exposed intermediate commits.
path = "src-tauri/src/spaces/nodes/insert.rs"
source = read(path)
start = source.index("    for node in nodes {", source.index("pub(in crate::spaces) async fn insert_many_in_transaction"))
end = source.index("async fn insert_single_node(", start)
source = source[:start] + r'''
    stage_nodes_in_transaction(tx, space_id, nodes, now).await?;
    complete_nodes_in_transaction(tx, space_id, nodes, now).await
}

pub(in crate::spaces) async fn stage_nodes_in_transaction(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, nodes: &[InsertNode], now: i64) -> Result<(), String> {
    for node in nodes { insert_single_node(tx, space_id, node, now).await?; }
    Ok(())
}
pub(in crate::spaces) async fn complete_nodes_in_transaction(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, nodes: &[InsertNode], now: i64) -> Result<(), String> {
    for node in nodes { contracts::validate_node(tx, space_id, &node.id, &node.kind, &node.data).await?; }
    for node in nodes { complete_single_node(tx, space_id, node, now).await?; }
    Ok(())
}

''' + source[end:]
source = source.replace('''    let object_type_id =
        contracts::validate_node(tx, space_id, &node.id, &node.kind, &node.data).await?;''', '''    if !crate::spaces::canvas_state::valid_structure_id(&node.id) { return Err("Invalid content identity".into()); }
    let object_type_id = if node.kind == "object" { Some(node.data.get("objectTypeId").and_then(serde_json::Value::as_str).ok_or("Objects require a type")?) } else { None };''')
start = source.index("    events::append(", source.index("async fn insert_single_node("))
source = source[:start] + r'''
    Ok(())
}
async fn complete_single_node(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, node: &InsertNode, now: i64) -> Result<(), String> {
''' + source[start:]
write(path, source)
replace("src-tauri/src/spaces/nodes/mod.rs", "pub(in crate::spaces) use insert::insert_many_in_transaction;", "pub(in crate::spaces) use insert::{insert_many_in_transaction, stage_nodes_in_transaction, complete_nodes_in_transaction};")
replace("src-tauri/src/spaces/editing/mod.rs", "mod execution;", "mod execution;\nmod fragment_execution;\nmod fragment_types;")
path = "src-tauri/src/spaces/editing/execution.rs"
source = read(path)
source = source.replace("let before_state = if request.command.deletes_subtrees()", "let before_state = if request.command.changes_content()")
source = source.replace("let lifecycle = if request.command.deletes_subtrees()", "let lifecycle = if request.command.changes_content()")
source = source.replace("    let now = crate::clock::unix_time_millis()?;", r'''
    let now = crate::clock::unix_time_millis()?;
    let inserted = if let EditorCommand::InsertFragment { fragment, .. } = &request.command {
        super::fragment_execution::stage(&mut tx, &request.space_id, fragment, &plan.after, now).await?
    } else { vec![] };''')
source = source.replace("    if changed { nodes::touch_updated_at", "    if !inserted.is_empty() { nodes::complete_nodes_in_transaction(&mut tx, &request.space_id, &inserted, now).await?; }\n    if changed { nodes::touch_updated_at")
write(path, source)
# Navigation targets can refer to structure handles; knowledge node references remain content-only.
write("src-tauri/src/spaces/canvas_state/navigation.rs", r'''
use crate::spaces::db;

pub(in crate::spaces) async fn validate_item_target(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, id: &str) -> Result<(), String> {
    let (document, _) = super::load_in_transaction(tx, space_id).await?;
    if !document.structure.items.iter().any(|item| item.id == id) { return Err("Navigation target must be an active structural item in its Space".into()); }
    Ok(())
}
pub(in crate::spaces) async fn ensure_existing_space(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str) -> Result<(), String> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spaces WHERE id=$1)").bind(space_id).fetch_one(&mut **tx).await.map_err(db::sql_error)?;
    if !exists { return Err("The target Space does not exist".into()); }
    Ok(())
}
''')
replace("src-tauri/src/spaces/canvas_state/mod.rs", "pub(in crate::spaces) mod viewport;", "pub(in crate::spaces) mod viewport;\npub(in crate::spaces) mod navigation;")
replace("src-tauri/src/spaces/knowledge/node_contracts/projection/mod.rs", 'validate_same_space_node(tx, target_space_id, "", node_id).await?;', 'crate::spaces::canvas_state::navigation::validate_item_target(tx, target_space_id, node_id).await?;')
# Unit tests exercise the same pure native plan that executes in the transaction.
write("src-tauri/src/spaces/organization/fragment_tests.rs", r'''
use crate::spaces::canvas_state::{self, contract::SpaceEditorDocument};
use super::{fragment_contract::{EditorFragment, FragmentPlacement}, fragment_plan};

fn fragment() -> EditorFragment {
    serde_json::from_value(serde_json::json!({
        "structure":{"version":1,"items":[
            {"kind":"group","id":"group","title":"Selection","parentId":null,"order":0,"locked":false},
            {"kind":"content","id":"node","nodeId":"node","parentId":"group","order":0,"locked":false}
        ]},
        "layout":{"version":1,"items":[
            {"kind":"group","itemId":"group","transform":{"x":100,"y":200,"rotation":30,"scale":2}},
            {"kind":"content","itemId":"node","transform":{"x":25,"y":10,"rotation":0,"scale":1},"size":{"width":120,"height":80},"sizeAuthority":"frame"}
        ]},
        "nodes":[{"id":"node","kind":"note","data":{"kind":"note","title":"A","tiptapContent":{"type":"doc","content":[]},"plainText":""},"sourceChunks":[]}],
        "relations":[],"objectTypes":[],"relationStyles":[]
    })).expect("fragment")
}
fn placement() -> FragmentPlacement {
    serde_json::from_value(serde_json::json!({"parentId":null,"delta":{"x":24,"y":24}})).expect("placement")
}
#[test]
fn preserves_nested_local_geometry_and_moves_only_fragment_roots() {
    let source = fragment();
    let result = fragment_plan::plan(&SpaceEditorDocument::empty(), &source, &placement()).expect("plan");
    assert_eq!(result.after.layout.items[1], source.layout.items[1]);
    let world = canvas_state::geometry::project(&result.after).expect("geometry");
    assert!((world.world["group"].x - 124.0).abs() < 1e-9);
    assert_eq!(result.selection, vec!["group"]);
}
#[test]
fn rejects_missing_content_without_partially_mutating_the_destination() {
    let mut source = fragment(); source.nodes.clear();
    let target = SpaceEditorDocument::empty();
    assert!(fragment_plan::plan(&target,&source,&placement()).is_err());
    assert!(target.structure.items.is_empty());
}
#[test]
fn rejects_duplicate_structure_identity_in_the_target() {
    let source = fragment();
    let target = fragment_plan::plan(&SpaceEditorDocument::empty(),&source,&placement()).expect("first").after;
    assert!(fragment_plan::plan(&target,&source,&placement()).is_err());
}
''')
replace("src-tauri/src/spaces/organization/mod.rs", "mod fragment_plan;", "mod fragment_plan;\n#[cfg(test)] mod fragment_tests;")
for path, content in sources.items():
    destination = root / path
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(content)
marker.parent.mkdir(parents=True, exist_ok=True)
marker.write_text("Reviewed native fragment command integrated. Remove development artifacts before delivery.\n")
print(f"Applied native fragment insertion to {len(sources)} source files")
