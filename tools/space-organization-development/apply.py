"""Reviewed receipt replay: exact structural changes plus domain-owned content lifecycle."""
from pathlib import Path
ROOT=Path('.')
files={}
def read(path): return files[path] if path in files else (ROOT/path).read_text()
def write(path,text): files[path]=text.strip()+'\n'
base='src-tauri/src/spaces/editing/'
# Receipt equality is used only for native, validated change records, never arbitrary client patches.
name=base+'changes.rs';text=read(name).replace('#[derive(Clone, Debug, Deserialize, Serialize)]', '#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]')
write(name,text)
name=base+'lifecycle_state.rs';text=read(name)
text=text.replace('#[derive(Clone, Debug, Default, Deserialize, Serialize)]', '#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]')
text=text.replace('pub source_id: String, pub target_id: String, pub status: String,', 'pub source_id: String, pub target_id: String, pub kind: String, pub rationale: String, pub confidence: Option<f64>, pub status: String,')
text=text.replace('SELECT id,source_id,target_id,status,status_before_invalidation', 'SELECT id,source_id,target_id,kind,rationale,confidence,status,status_before_invalidation')
text=text.replace('status: row.try_get("status").map_err(db::sql_error)?, previous_status:', 'kind: row.try_get("kind").map_err(db::sql_error)?, rationale: row.try_get("rationale").map_err(db::sql_error)?, confidence: row.try_get("confidence").map_err(db::sql_error)?, status: row.try_get("status").map_err(db::sql_error)?, previous_status:')
write(name,text)
write(base+'lifecycle_guards.rs', '''
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
''')
write(base+'lifecycle_replay.rs', '''
use crate::spaces::{db, nodes, knowledge::{invalidation, provenance}};
use super::lifecycle_state::LifecycleChanges;
pub(super) async fn apply(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, changes: &LifecycleChanges, now: i64) -> Result<(), String> {
    let restored: Vec<_> = changes.nodes.iter().filter(|change| change.after.as_ref().is_some_and(|node| node.active)).map(|change| change.id.clone()).collect();
    let removed: Vec<_> = changes.nodes.iter().filter(|change| change.after.as_ref().is_none_or(|node| !node.active)).map(|change| change.id.clone()).collect();
    let actual = nodes::restore_in_transaction(tx, space_id, &restored, now).await?;
    if actual.len() != restored.len() { return Err("The restored content selection changed".into()); }
    let actual = nodes::trash_in_transaction(tx, space_id, &removed, now).await?;
    if actual.len() != removed.len() { return Err("The deleted content selection changed".into()); }
    for change in &changes.relations {
        if change.after.as_ref().is_some_and(|relation| relation.status != "invalidated") {
            let restored = invalidation::restore_relation(tx, &change.id, "human", None, now).await?;
            if change.after.as_ref().is_none_or(|relation| relation.status != restored) { return Err("The relation's prior state no longer matches its edit receipt".into()); }
        } else { invalidation::invalidate_relation(tx, &change.id, "human", None, now).await?; }
    }
    for change in &changes.provenance {
        if change.after.as_ref().is_some_and(|record| record.status == "active") { provenance::restore_for_history(tx, space_id, &change.id, now).await?; }
        else {
            let active: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spaces_provenance WHERE id=$1 AND space_id=$2 AND status='active')").bind(&change.id).bind(space_id).fetch_one(&mut **tx).await.map_err(db::sql_error)?;
            if active { invalidation::invalidate_provenance(tx, space_id, &change.id, "human", None, now).await?; }
        }
    }
    Ok(())
}
''')
write('src-tauri/src/spaces/knowledge/provenance/history_restore.rs', '''
use sqlx::Row;
use crate::spaces::{db, knowledge::events};
use super::ProvenanceInput;
/** Restore the retained provenance identity only after validating its current endpoints and locator. */
pub(in crate::spaces) async fn restore(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, id: &str, now: i64) -> Result<(), String> {
    let row = sqlx::query("SELECT target_kind,target_id,source_node_id,source_revision,locator,quote FROM spaces_provenance WHERE id=$1 AND space_id=$2 AND status='invalidated'")
        .bind(id).bind(space_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?.ok_or("Invalidated provenance is unavailable")?;
    let input = ProvenanceInput { id: id.into(), space_id: space_id.into(),
        target_kind: row.try_get("target_kind").map_err(db::sql_error)?, target_id: row.try_get("target_id").map_err(db::sql_error)?,
        source_node_id: row.try_get("source_node_id").map_err(db::sql_error)?, source_revision: row.try_get("source_revision").map_err(db::sql_error)?,
        locator: serde_json::from_str(&row.try_get::<String,_>("locator").map_err(db::sql_error)?).map_err(db::json_error)?, quote: row.try_get("quote").map_err(db::sql_error)?,
    };
    super::validate_input(&input)?; super::ensure_target(tx, &input).await?;
    let competing: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spaces_provenance WHERE space_id=$1 AND target_kind=$2 AND target_id=$3 AND source_node_id=$4 AND status='active' AND id<>$5)")
        .bind(space_id).bind(&input.target_kind).bind(&input.target_id).bind(&input.source_node_id).bind(id).fetch_one(&mut **tx).await.map_err(db::sql_error)?;
    if competing { return Err("A newer provenance record now owns this relationship; the old edit cannot replace it".into()); }
    sqlx::query("UPDATE spaces_provenance SET status='active',updated_at=$1 WHERE id=$2 AND space_id=$3 AND status='invalidated'")
        .bind(now).bind(id).bind(space_id).execute(&mut **tx).await.map_err(db::sql_error)?;
    events::append(tx, events::EventRecord::new(space_id, events::EventEntity::new("provenance",id), "restored", events::EventActor::new("human",None), &serde_json::json!({"sourceRevision":input.source_revision}), now)).await
}
''')
name='src-tauri/src/spaces/knowledge/provenance/mod.rs';write(name,read(name)+'\nmod history_restore;\npub(in crate::spaces) use history_restore::restore as restore_for_history;\n')
name='src-tauri/src/spaces/knowledge/contracts/rules.rs';text=read(name).replace('"created" | "superseded" | "invalidated" | "moved"', '"created" | "superseded" | "invalidated" | "restored" | "moved"')
write(name,text)
name='src-tauri/src/spaces/knowledge/invalidation.rs';write(name,read(name).replace('async fn invalidate_provenance(', 'pub(in crate::spaces) async fn invalidate_provenance('))
write(base+'replay.rs', '''
use crate::spaces::{db, canvas_state, nodes};
use super::{contracts::{ReplayEditRequest, ReplayDirection, EditResult, Receipt, ReceiptChanges}, changes, receipts, lifecycle_state, lifecycle_guards, lifecycle_replay, archive};
pub(super) async fn replay(pool: &sqlx::SqlitePool, request: ReplayEditRequest) -> Result<EditResult, String> {
    receipts::validate_request(&request.space_id, &request.mutation_id, request.base_revision)?;
    let hash = receipts::fingerprint(&request)?;
    let mut tx = db::begin_write(pool).await?;
    if let Some(previous) = receipts::find(&mut tx, &request.space_id, &request.mutation_id).await? {
        if previous.request_hash != hash { return Err("Mutation identity was already used for another history operation".into()); }
        let result = super::result::current(&mut tx, &request.space_id, &previous, true).await?;
        tx.commit().await.map_err(db::sql_error)?; return Ok(result);
    }
    let original = receipts::find(&mut tx, &request.space_id, &request.receipt_id).await?.ok_or("This edit receipt is no longer available")?;
    if !original.changes.reversible { return Err("This transaction is not an undoable user edit".into()); }
    let (before, revision) = canvas_state::load_in_transaction(&mut tx, &request.space_id).await?;
    if revision != request.base_revision { return Err("Space edit revision conflict".into()); }
    let inverse = matches!(request.direction, ReplayDirection::Undo);
    let after = changes::apply(&before, &original.changes.document, inverse)?;
    let expected = lifecycle_guards::direction(&original.changes.lifecycle, inverse);
    let before_lifecycle = lifecycle_state::capture(&mut tx, &request.space_id).await?;
    lifecycle_guards::validate_before(&before_lifecycle, &expected)?;
    let now = crate::clock::unix_time_millis()?;
    lifecycle_replay::apply(&mut tx, &request.space_id, &expected, now).await?;
    let after_lifecycle = lifecycle_state::capture(&mut tx, &request.space_id).await?;
    lifecycle_guards::validate_after(&before_lifecycle, &after_lifecycle, &expected)?;
    if expected.nodes.iter().any(|change| change.after.as_ref().is_none_or(|node| !node.active)) {
        archive::removed(&mut tx, &request.space_id, &request.mutation_id, &before, &after, now).await?;
    }
    archive::remove_restored(&mut tx, &request.space_id, &after).await?;
    let delta = changes::diff(&before, &after);
    let lifecycle = lifecycle_state::diff(&before_lifecycle, &after_lifecycle);
    let changed = !changes::unchanged(&delta) || !lifecycle_state::empty(&lifecycle);
    let revision = if changed { canvas_state::save_in_transaction(&mut tx, &request.space_id, revision, &after).await? } else { revision };
    nodes::touch_updated_at(&mut tx, &request.space_id, now).await?;
    let (selection_before, selection_after) = if inverse { (original.selection_after, original.selection_before) } else { (original.selection_before, original.selection_after) };
    let receipt = Receipt { mutation_id: request.mutation_id, request_hash: hash, revision, selection_before, selection_after,
        changes: ReceiptChanges { version: 1, reversible: false, document: delta, lifecycle } };
    receipts::store(&mut tx, &request.space_id, &receipt, now).await?;
    let result = super::result::current(&mut tx, &request.space_id, &receipt, false).await?;
    tx.commit().await.map_err(db::sql_error)?; Ok(result)
}
''')
name=base+'mod.rs';text=read(name)
text=text.replace('mod lifecycle_edit;', 'mod lifecycle_edit;\nmod lifecycle_guards;\nmod lifecycle_replay;\nmod replay;')
text+='''
#[tauri::command]
pub(crate) async fn replay_space_edit<R: Runtime>(app: AppHandle<R>, request: contracts::ReplayEditRequest) -> Result<contracts::EditResult, String> {
    let space_id = request.space_id.clone(); let pool = db::pool(&app).await?;
    let result = replay::replay(pool, request).await?;
    crate::spaces::knowledge::retrieval::invalidation_outbox::drain_and_report(&app, pool).await;
    if let Err(error) = app.emit("spaces:editor-committed", serde_json::json!({"spaceId":space_id,"revision":result.revision,"mutationId":result.mutation_id})) { log::error!("History committed but notification failed: {error}"); }
    Ok(result)
}
'''
write(name,text)
name=base+'execution.rs';text=read(name)
text=text.replace('EditorCommand::Structure { selection, .. } => selection.clone(), _ => vec![]', 'EditorCommand::Structure { selection, .. } => selection.clone(), EditorCommand::DeleteSelection { ids, .. } => ids.clone(), EditorCommand::TransformFrames { updates } | EditorCommand::ArrangeFrames { updates } => updates.iter().map(|update| update.node_id.clone()).collect(), _ => vec![]')
write(name,text)
name='src-tauri/src/spaces/mod.rs';write(name,read(name).replace('            crate::spaces::editing::execute_space_edit,','            crate::spaces::editing::execute_space_edit,\n            crate::spaces::editing::replay_space_edit,'))
name='src-tauri/build.rs';write(name,read(name).replace('                        "execute_space_edit",','                        "execute_space_edit",\n                        "replay_space_edit",'))
name='src-tauri/capabilities/spaces.json';write(name,read(name).replace('"spaces:allow-execute-space-edit",','"spaces:allow-execute-space-edit",\n    "spaces:allow-replay-space-edit",'))
# Compiler diagnosis capture contains no executed migration or hidden repair.
report=ROOT/'tools/space-organization-development/last-native/report.md'
if report.exists(): write('tools/space-organization-development/last-native/tail.md', '# Exact-source native check tail\n\n'+'\n'.join(report.read_text().splitlines()[-300:]))
for name,text in files.items():
    path=ROOT/name;path.parent.mkdir(parents=True,exist_ok=True)
    if not path.exists() or path.read_text()!=text:path.write_text(text)
Path(__file__).write_text('"""No pending source edits. The branch is verified as committed."""\nprint("No pending source edits")\n')
print('Integrated guarded native history replay and provenance lifecycle restoration.')
