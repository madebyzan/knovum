"""Reviewed native editor transaction, exact receipts and coherent workspace reads."""
from pathlib import Path
import re
ROOT=Path('.')
files={}
def read(path): return files[path] if path in files else (ROOT/path).read_text()
def write(path,text): files[path]=text.strip()+'\n'
base='src-tauri/src/spaces/editing/'
write(base+'contracts.rs', '''
use serde::{Deserialize, Serialize};
use crate::spaces::{organization::EditorCommand, canvas_state::contract::SpaceEditorDocument};
use super::{changes::EditChanges, lifecycle_state::LifecycleChanges};
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ExecuteEditRequest { pub space_id: String, pub mutation_id: String, pub base_revision: i64, pub command: EditorCommand }
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ReplayEditRequest { pub space_id: String, pub mutation_id: String, pub base_revision: i64, pub receipt_id: String, pub direction: ReplayDirection }
#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ReplayDirection { Undo, Redo }
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ReceiptQuery { pub space_id: String, pub mutation_id: String }
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::spaces) struct ReceiptChanges { pub version: u8, pub reversible: bool, pub document: EditChanges, pub lifecycle: LifecycleChanges }
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EditResult {
    pub document: SpaceEditorDocument, pub revision: i64, pub applied_at_revision: i64, pub mutation_id: String,
    pub selection: Vec<String>, pub history_receipt_id: Option<String>,
    pub content: Option<crate::spaces::queries::EditorWorkspaceContent>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EditorWorkspace { pub document: SpaceEditorDocument, pub revision: i64, pub content: crate::spaces::queries::EditorWorkspaceContent }
pub(super) struct Receipt {
    pub mutation_id: String, pub request_hash: String, pub revision: i64, pub changes: ReceiptChanges,
    pub selection_before: Vec<String>, pub selection_after: Vec<String>,
}
''')
write(base+'lifecycle_state.rs', '''
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use crate::spaces::db;
use super::changes::RecordChange;
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct RelationState { pub id: String, pub source_id: String, pub target_id: String, pub status: String, pub previous_status: Option<String> }
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ProvenanceState {
    pub id: String, pub target_kind: String, pub target_id: String, pub source_node_id: String,
    pub source_revision: String, pub locator: Value, pub quote: String, pub status: String,
}
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct NodeActivity { pub id: String, pub active: bool }
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::spaces) struct LifecycleChanges {
    pub(super) nodes: Vec<RecordChange<NodeActivity>>,
    pub(super) relations: Vec<RecordChange<RelationState>>,
    pub(super) provenance: Vec<RecordChange<ProvenanceState>>,
}
#[derive(Default)]
pub(super) struct LifecycleState {
    pub nodes: BTreeMap<String, NodeActivity>, pub relations: BTreeMap<String, RelationState>, pub provenance: BTreeMap<String, ProvenanceState>,
}
pub(super) async fn capture(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str) -> Result<LifecycleState, String> {
    let mut state = LifecycleState::default();
    for row in sqlx::query("SELECT id,deleted_at FROM spaces_nodes WHERE space_id=$1").bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)? {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        state.nodes.insert(id.clone(), NodeActivity { id, active: row.try_get::<Option<i64>,_>("deleted_at").map_err(db::sql_error)?.is_none() });
    }
    for row in sqlx::query("SELECT id,source_id,target_id,status,status_before_invalidation FROM spaces_relations WHERE space_id=$1").bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)? {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        state.relations.insert(id.clone(), RelationState { id, source_id: row.try_get("source_id").map_err(db::sql_error)?, target_id: row.try_get("target_id").map_err(db::sql_error)?, status: row.try_get("status").map_err(db::sql_error)?, previous_status: row.try_get("status_before_invalidation").map_err(db::sql_error)? });
    }
    for row in sqlx::query("SELECT id,target_kind,target_id,source_node_id,source_revision,locator,quote,status FROM spaces_provenance WHERE space_id=$1").bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)? {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        state.provenance.insert(id.clone(), ProvenanceState { id, target_kind: row.try_get("target_kind").map_err(db::sql_error)?, target_id: row.try_get("target_id").map_err(db::sql_error)?, source_node_id: row.try_get("source_node_id").map_err(db::sql_error)?, source_revision: row.try_get("source_revision").map_err(db::sql_error)?, locator: serde_json::from_str(&row.try_get::<String,_>("locator").map_err(db::sql_error)?).map_err(db::json_error)?, quote: row.try_get("quote").map_err(db::sql_error)?, status: row.try_get("status").map_err(db::sql_error)? });
    }
    Ok(state)
}
pub(super) fn diff(before: &LifecycleState, after: &LifecycleState) -> LifecycleChanges {
    LifecycleChanges { nodes: differences(&before.nodes, &after.nodes), relations: differences(&before.relations, &after.relations), provenance: differences(&before.provenance, &after.provenance) }
}
fn differences<T: Clone + PartialEq>(before: &BTreeMap<String,T>, after: &BTreeMap<String,T>) -> Vec<RecordChange<T>> {
    let keys: std::collections::BTreeSet<_> = before.keys().chain(after.keys()).cloned().collect();
    keys.into_iter().filter_map(|id| { let left = before.get(&id); let right = after.get(&id); if left == right { None } else { Some(RecordChange { id, before: left.cloned(), after: right.cloned() }) } }).collect()
}
pub(in crate::spaces) fn empty(changes: &LifecycleChanges) -> bool { changes.nodes.is_empty() && changes.relations.is_empty() && changes.provenance.is_empty() }
''')
write(base+'archive.rs', '''
use std::collections::{HashMap, HashSet};
use crate::spaces::{db, canvas_state::{self, contract::SpaceEditorDocument}};
pub(super) async fn removed(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, batch_id: &str, before: &SpaceEditorDocument, after: &SpaceEditorDocument, now: i64) -> Result<(), String> {
    let retained: HashSet<&str> = after.structure.items.iter().map(|item| item.id.as_str()).collect();
    let layouts: HashMap<_,_> = before.layout.items.iter().map(|layout| (layout.id(),layout)).collect();
    let projection = canvas_state::geometry::project(before)?;
    for item in &before.structure.items {
        if retained.contains(item.id.as_str()) { continue; }
        sqlx::query("INSERT INTO spaces_editor_trash(space_id,item_id,batch_id,structure,layout,world_transform,deleted_at) VALUES($1,$2,$3,$4,$5,$6,$7) ON CONFLICT(space_id,item_id) DO UPDATE SET batch_id=excluded.batch_id,structure=excluded.structure,layout=excluded.layout,world_transform=excluded.world_transform,deleted_at=excluded.deleted_at")
            .bind(space_id).bind(&item.id).bind(batch_id).bind(serde_json::to_string(item).map_err(db::json_error)?).bind(serde_json::to_string(layouts.get(item.id.as_str()).ok_or("Missing deleted layout")?).map_err(db::json_error)?)
            .bind(serde_json::to_string(projection.world.get(&item.id).ok_or("Missing deleted transform")?).map_err(db::json_error)?).bind(now).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}
pub(super) async fn remove_restored(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, after: &SpaceEditorDocument) -> Result<(), String> {
    let ids: Vec<_> = after.structure.items.iter().map(|item| &item.id).collect();
    sqlx::query("DELETE FROM spaces_editor_trash WHERE space_id=$1 AND item_id IN (SELECT value FROM json_each($2))").bind(space_id).bind(serde_json::to_string(&ids).map_err(db::json_error)?).execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}
''')
write(base+'receipts.rs', '''
use serde::Serialize;
use sqlx::Row;
use crate::spaces::db;
use super::contracts::{Receipt, ReceiptChanges};
pub(super) fn validate_request(space_id: &str, mutation_id: &str, revision: i64) -> Result<(), String> {
    if !crate::spaces::canvas_state::valid_structure_id(space_id) || uuid::Uuid::parse_str(mutation_id).is_err() || !(0..9_007_199_254_740_991).contains(&revision) { return Err("Invalid editor identity, mutation identity or revision".into()); }
    Ok(())
}
pub(super) fn fingerprint(value: &impl Serialize) -> Result<String, String> {
    let bytes = serde_json::to_vec(value).map_err(db::json_error)?;
    if bytes.len() > 128 * 1024 * 1024 { return Err("Editor request exceeds its transfer limit".into()); }
    let digest = ring::digest::digest(&ring::digest::SHA256, &bytes);
    Ok(digest.as_ref().iter().map(|byte| format!("{byte:02x}")).collect())
}
pub(super) async fn find(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, mutation_id: &str) -> Result<Option<Receipt>, String> {
    let row = sqlx::query("SELECT mutation_id,request_hash,applied_revision,changes,selection_before,selection_after,invalidated_at FROM spaces_edit_receipts WHERE space_id=$1 AND mutation_id=$2")
        .bind(space_id).bind(mutation_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?;
    let Some(row) = row else { return Ok(None); };
    if row.try_get::<Option<i64>,_>("invalidated_at").map_err(db::sql_error)?.is_some() { return Err("This edit refers to permanently removed data and cannot be replayed".into()); }
    let changes: ReceiptChanges = serde_json::from_str(&row.try_get::<String,_>("changes").map_err(db::sql_error)?).map_err(db::json_error)?;
    if changes.version != 1 { return Err("Unsupported edit receipt".into()); }
    Ok(Some(Receipt { mutation_id: row.try_get("mutation_id").map_err(db::sql_error)?, request_hash: row.try_get("request_hash").map_err(db::sql_error)?, revision: row.try_get("applied_revision").map_err(db::sql_error)?, changes,
        selection_before: serde_json::from_str(&row.try_get::<String,_>("selection_before").map_err(db::sql_error)?).map_err(db::json_error)?,
        selection_after: serde_json::from_str(&row.try_get::<String,_>("selection_after").map_err(db::sql_error)?).map_err(db::json_error)?,
    }))
}
pub(super) async fn store(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, receipt: &Receipt, now: i64) -> Result<(), String> {
    sqlx::query("INSERT INTO spaces_edit_receipts(space_id,mutation_id,request_hash,applied_revision,changes,selection_before,selection_after,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
        .bind(space_id).bind(&receipt.mutation_id).bind(&receipt.request_hash).bind(receipt.revision)
        .bind(serde_json::to_string(&receipt.changes).map_err(db::json_error)?).bind(serde_json::to_string(&receipt.selection_before).map_err(db::json_error)?).bind(serde_json::to_string(&receipt.selection_after).map_err(db::json_error)?).bind(now)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}
''')
write(base+'measurements.rs', '''
use sqlx::Row;
use serde_json::Value;
use crate::spaces::{db, organization::TextMeasurement, canvas_state::contract::{SpaceEditorDocument, StructureKind, StructureLayout}};
pub(super) async fn apply(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, before: &SpaceEditorDocument, measurements: &[TextMeasurement]) -> Result<SpaceEditorDocument, String> {
    if measurements.len() > 10_000 { return Err("Too many content measurements".into()); }
    let mut after = before.clone();
    let mut seen = std::collections::HashSet::new();
    for measurement in measurements {
        if !seen.insert(&measurement.item_id) || !measurement.width.is_finite() || !measurement.height.is_finite() || measurement.width <= 0.0 || measurement.height <= 0.0 { return Err("Invalid content measurement".into()); }
        let item = before.structure.items.iter().find(|item| item.id == measurement.item_id);
        let Some(item) = item else { continue; };
        if item.kind != StructureKind::Content { return Err("Only content can report a text measurement".into()); }
        let row = sqlx::query("SELECT kind,data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL").bind(item.node_id.as_deref()).bind(space_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?;
        let Some(row) = row else { continue; };
        if row.try_get::<String,_>("kind").map_err(db::sql_error)? != "text" { return Err("Only Text uses content-fit height".into()); }
        let data: Value = serde_json::from_str(&row.try_get::<String,_>("data").map_err(db::sql_error)?).map_err(db::json_error)?;
        if crate::spaces::knowledge::indexing::revision::render_revision(&data) != measurement.content_revision { continue; }
        if let Some(StructureLayout::Content { size, appearance, .. }) = after.layout.items.iter_mut().find(|layout| layout.id() == item.id) {
            let scale = appearance.as_ref().and_then(|appearance| appearance.typography_scale).unwrap_or(1.0);
            if (size.width - measurement.width).abs() > 0.5 || scale != measurement.typography_scale { continue; }
            size.height = measurement.height.ceil();
        }
    }
    crate::spaces::canvas_state::validate_document(&after)?;
    Ok(after)
}
''')
write(base+'execution.rs', '''
use crate::spaces::{db, canvas_state, organization::{self, EditorCommand}, nodes};
use super::{contracts::{ExecuteEditRequest, EditResult, Receipt, ReceiptChanges}, changes, receipts, lifecycle_state, archive};
pub(super) async fn execute(pool: &sqlx::SqlitePool, request: ExecuteEditRequest) -> Result<EditResult, String> {
    receipts::validate_request(&request.space_id, &request.mutation_id, request.base_revision)?;
    let hash = receipts::fingerprint(&request)?;
    let mut tx = db::begin_write(pool).await?;
    if let Some(receipt) = receipts::find(&mut tx, &request.space_id, &request.mutation_id).await? {
        if receipt.request_hash != hash { return Err("A mutation identity cannot be reused for a different edit".into()); }
        let result = super::result::current(&mut tx, &request.space_id, &receipt, true).await?;
        tx.commit().await.map_err(db::sql_error)?; return Ok(result);
    }
    let (before, revision) = canvas_state::load_in_transaction(&mut tx, &request.space_id).await?;
    canvas_state::validation::validate_content_references(&mut tx, &request.space_id, &before).await?;
    if revision != request.base_revision { return Err("Space edit revision conflict".into()); }
    let before_state = if request.command.deletes_subtrees() { lifecycle_state::capture(&mut tx, &request.space_id).await? } else { lifecycle_state::LifecycleState::default() };
    let mut plan = match &request.command {
        EditorCommand::MeasureText { measurements } => organization::Plan { after: super::measurements::apply(&mut tx, &request.space_id, &before, measurements).await?, selection: vec![], removed_content_ids: vec![] },
        _ => organization::plan_editor(&before, &request.command)?,
    };
    let now = crate::clock::unix_time_millis()?;
    if request.command.deletes_subtrees() {
        super::lifecycle_edit::delete(&mut tx, &request.space_id, &plan.removed_content_ids, request.command.requested_relation_deletions(), now).await?;
        archive::removed(&mut tx, &request.space_id, &request.mutation_id, &before, &plan.after, now).await?;
        let live: Vec<String> = sqlx::query_scalar("SELECT id FROM spaces_relations WHERE space_id=$1 AND status<>'invalidated'").bind(&request.space_id).fetch_all(&mut *tx).await.map_err(db::sql_error)?;
        if let Some(styles) = plan.after.relation_styles.as_mut() { styles.retain(|style| live.contains(&style.relation_id)); }
    }
    let lifecycle = if request.command.deletes_subtrees() { lifecycle_state::diff(&before_state, &lifecycle_state::capture(&mut tx, &request.space_id).await?) } else { Default::default() };
    let delta = changes::diff(&before, &plan.after);
    let changed = !changes::unchanged(&delta) || !lifecycle_state::empty(&lifecycle);
    let revision = if changed { canvas_state::save_in_transaction(&mut tx, &request.space_id, revision, &plan.after).await? } else { revision };
    if changed { nodes::touch_updated_at(&mut tx, &request.space_id, now).await?; }
    let selection_before = match &request.command { EditorCommand::Structure { selection, .. } => selection.clone(), _ => vec![] };
    let receipt = Receipt { mutation_id: request.mutation_id, request_hash: hash, revision, selection_before, selection_after: plan.selection,
        changes: ReceiptChanges { version: 1, reversible: changed && !matches!(request.command, EditorCommand::MeasureText { .. }), document: delta, lifecycle },
    };
    receipts::store(&mut tx, &request.space_id, &receipt, now).await?;
    let result = super::result::current(&mut tx, &request.space_id, &receipt, false).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(result)
}
''')
write(base+'lifecycle_edit.rs', '''
use crate::spaces::{db, knowledge::invalidation, nodes};
pub(super) async fn delete(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, content_ids: &[String], relation_ids: &[String], now: i64) -> Result<(), String> {
    let mut relations = std::collections::HashSet::new();
    for id in relation_ids {
        if !relations.insert(id) { return Err("Duplicate relation deletion identity".into()); }
        let belongs: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spaces_relations WHERE id=$1 AND space_id=$2 AND status<>'invalidated')").bind(id).bind(space_id).fetch_one(&mut **tx).await.map_err(db::sql_error)?;
        if !belongs { return Err("The selected relation is no longer active in this Space".into()); }
    }
    let removed = nodes::trash_in_transaction(tx, space_id, content_ids, now).await?;
    if removed.len() != content_ids.len() { return Err("Content deletion selection changed".into()); }
    for id in relation_ids { invalidation::invalidate_relation(tx, id, "human", None, now).await?; }
    Ok(())
}
''')
write(base+'result.rs', '''
use std::collections::HashSet;
use crate::spaces::{canvas_state, queries};
use super::{contracts::{Receipt, EditResult}, lifecycle_state};
pub(super) async fn current(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, receipt: &Receipt, replayed: bool) -> Result<EditResult, String> {
    let (document, revision) = canvas_state::load_in_transaction(tx, space_id).await?;
    canvas_state::validation::validate_content_references(tx, space_id, &document).await?;
    let ids: HashSet<_> = document.structure.items.iter().map(|item| item.id.as_str()).collect();
    let selection = receipt.selection_after.iter().filter(|id| ids.contains(id.as_str())).cloned().collect();
    let content = if replayed || !lifecycle_state::empty(&receipt.changes.lifecycle) { Some(queries::read_editor_content(tx, space_id).await?) } else { None };
    Ok(EditResult { document, revision, applied_at_revision: receipt.revision, mutation_id: receipt.mutation_id.clone(), selection,
        history_receipt_id: receipt.changes.reversible.then(|| receipt.mutation_id.clone()), content })
}
''')
write('src-tauri/src/spaces/queries/editor_workspace.rs', '''
use serde::Serialize;
use serde_json::Value;
use crate::spaces::db;
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EditorWorkspaceContent { pub nodes: Vec<Value>, pub relations: Vec<Value> }
pub(in crate::spaces) async fn read_content(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str) -> Result<EditorWorkspaceContent, String> {
    let nodes = sqlx::query("SELECT id,initial_x,initial_y,data FROM spaces_nodes WHERE space_id=$1 AND deleted_at IS NULL ORDER BY created_at,id")
        .bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?.into_iter().map(super::nodes::node_row).collect::<Result<_,_>>()?;
    let relations = sqlx::query("SELECT id,space_id,source_id,target_id,kind,rationale,status,status_before_invalidation,confidence,actor_kind,actor_id,created_at,updated_at FROM spaces_relations WHERE space_id=$1 AND status<>'invalidated' ORDER BY created_at,id")
        .bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?.into_iter().map(super::relations::relation_row).collect::<Result<_,_>>()?;
    Ok(EditorWorkspaceContent { nodes, relations })
}
''')
for name,function in [('src-tauri/src/spaces/queries/nodes.rs','node_row'),('src-tauri/src/spaces/queries/relations.rs','relation_row')]:
    write(name,read(name).replace(f'fn {function}(',f'pub(super) fn {function}('))
name='src-tauri/src/spaces/queries/mod.rs';write(name,read(name)+'\nmod editor_workspace;\npub(crate) use editor_workspace::EditorWorkspaceContent;\npub(in crate::spaces) use editor_workspace::read_content as read_editor_content;\n')
name='src-tauri/src/spaces/canvas_state/mod.rs';write(name,read(name).replace('mod validation;', 'pub(in crate::spaces) mod validation;'))
# Rendering revisions are independent of whether a Text item participates in knowledge indexing.
name='src-tauri/src/spaces/knowledge/indexing/revision/mod.rs';text=read(name)
text+='''
pub(in crate::spaces) fn render_revision(data: &Value) -> String { format!("render1:{:016x}", fnv1a64(stable_json(data).as_bytes())) }
'''
write(name,text)
# Preserve Saved View identity references through undoable content deletion.
name='src-tauri/src/spaces/nodes/lifecycle/effects.rs';text=read(name)
text=text.replace('    remove_nodes_from_scene_focus(tx, space_id, ids, now).await?;\n','')
text=re.sub(r'pub\(super\) async fn remove_nodes_from_scene_focus\([\s\S]*?\n}\n\n','',text,count=1)
write(name,text)
write(base+'mod.rs', '''
pub(in crate::spaces) mod changes;
mod contracts;
mod lifecycle_state;
mod archive;
mod receipts;
mod measurements;
mod execution;
mod lifecycle_edit;
mod result;
use tauri::{AppHandle, Emitter, Runtime};
use crate::spaces::{db, canvas_state, queries};
#[tauri::command]
pub(crate) async fn execute_space_edit<R: Runtime>(app: AppHandle<R>, request: contracts::ExecuteEditRequest) -> Result<contracts::EditResult, String> {
    let space_id = request.space_id.clone();
    let pool = db::pool(&app).await?;
    let result = execution::execute(pool, request).await?;
    crate::spaces::knowledge::retrieval::invalidation_outbox::drain_and_report(&app, pool).await;
    if let Err(error) = app.emit("spaces:editor-committed", serde_json::json!({"spaceId": space_id, "revision": result.revision, "mutationId": result.mutation_id})) { log::error!("Editor committed but notification failed: {error}"); }
    Ok(result)
}
#[tauri::command]
pub(crate) async fn query_space_edit_receipt<R: Runtime>(app: AppHandle<R>, request: contracts::ReceiptQuery) -> Result<Option<contracts::EditResult>, String> {
    let pool = db::pool(&app).await?; let mut tx = pool.begin().await.map_err(db::sql_error)?;
    let receipt = receipts::find(&mut tx, &request.space_id, &request.mutation_id).await?;
    let result = match receipt { Some(receipt) => Some(result::current(&mut tx, &request.space_id, &receipt, true).await?), None => None };
    tx.commit().await.map_err(db::sql_error)?; Ok(result)
}
#[tauri::command]
pub(crate) async fn load_editor_workspace<R: Runtime>(app: AppHandle<R>, space_id: String) -> Result<contracts::EditorWorkspace, String> {
    let pool = db::pool(&app).await?; let mut tx = pool.begin().await.map_err(db::sql_error)?;
    let (document, revision) = canvas_state::load_in_transaction(&mut tx, &space_id).await?;
    canvas_state::validation::validate_content_references(&mut tx, &space_id, &document).await?;
    let content = queries::read_editor_content(&mut tx, &space_id).await?;
    tx.commit().await.map_err(db::sql_error)?; Ok(contracts::EditorWorkspace { document, revision, content })
}
''')
# Register only finite editing and read/query boundaries. Legacy write commands are removed after the remaining creation/lifecycle adapters are cut over.
name='src-tauri/src/spaces/mod.rs';text=read(name)
text=text.replace('            crate::spaces::canvas_state::save_editor_viewport,','            crate::spaces::canvas_state::save_editor_viewport,\n            crate::spaces::editing::execute_space_edit,\n            crate::spaces::editing::query_space_edit_receipt,\n            crate::spaces::editing::load_editor_workspace,')
write(name,text)
name='src-tauri/build.rs';text=read(name).replace('                        "save_editor_viewport",','                        "save_editor_viewport",\n                        "execute_space_edit",\n                        "query_space_edit_receipt",\n                        "load_editor_workspace",')
write(name,text)
name='src-tauri/capabilities/spaces.json';text=read(name)
text=text.replace('"spaces:allow-save-editor-viewport",','"spaces:allow-save-editor-viewport",\n    "spaces:allow-execute-space-edit",\n    "spaces:allow-query-space-edit-receipt",\n    "spaces:allow-load-editor-workspace",')
write(name,text)
for name,text in files.items():
    path=ROOT/name;path.parent.mkdir(parents=True,exist_ok=True)
    if not path.exists() or path.read_text()!=text:path.write_text(text)
Path(__file__).write_text('"""No pending source edits. The branch is verified as committed."""\nprint("No pending source edits")\n')
print('Implemented the authoritative editor transaction, exact receipts and coherent workspace reads.')
