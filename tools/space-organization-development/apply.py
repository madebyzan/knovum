"""Reviewed additive camera/receipt contracts, retaining one strict JSON organization authority.
This changes source schema for fresh development installations; it never opens a user database.
"""
from pathlib import Path
import sqlite3
import re
ROOT=Path('.')
files={}
def read(path): return files[path] if path in files else (ROOT/path).read_text()
def write(path,text): files[path]=text.strip()+'\n'
name='src-tauri/migrations/spaces/0001_spaces.sql';schema=read(name)
# The resource index identifies structural items, not fictitious knowledge nodes.
start=schema.index('CREATE TABLE spaces_canvas_asset_references (')
end=schema.index('CREATE VIEW spaces_semantic_nodes',start)
schema=schema[:start]+'''CREATE TABLE spaces_canvas_asset_references (
  space_id TEXT NOT NULL,
  item_id TEXT NOT NULL,
  path TEXT NOT NULL CHECK (length(path) > 0),
  PRIMARY KEY (space_id, item_id),
  FOREIGN KEY (space_id) REFERENCES spaces(id) ON DELETE CASCADE
);
CREATE TRIGGER spaces_canvas_asset_reference_same_space_insert
BEFORE INSERT ON spaces_canvas_asset_references BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM spaces_canvas_state c, json_each(c.document,'$.layout.items') l
    WHERE c.space_id=NEW.space_id AND json_extract(l.value,'$.itemId')=NEW.item_id
      AND json_extract(l.value,'$.appearance.background')=NEW.path
  ) THEN RAISE(ABORT,'Background reference must match its owning editor layout') END;
END;
CREATE TRIGGER spaces_canvas_asset_reference_same_space_update
BEFORE UPDATE ON spaces_canvas_asset_references BEGIN
  SELECT CASE WHEN NOT EXISTS (
    SELECT 1 FROM spaces_canvas_state c, json_each(c.document,'$.layout.items') l
    WHERE c.space_id=NEW.space_id AND json_extract(l.value,'$.itemId')=NEW.item_id
      AND json_extract(l.value,'$.appearance.background')=NEW.path
  ) THEN RAISE(ABORT,'Background reference must match its owning editor layout') END;
END;

'''+schema[end:]
schema+='''

-- Camera movement is not an organizational edit and does not increment edit revision.
CREATE TABLE spaces_canvas_viewports (
  space_id TEXT PRIMARY KEY NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
  viewport TEXT NOT NULL CHECK (json_valid(viewport)),
  updated_at INTEGER NOT NULL
);
-- Bounded reversible edit receipts. Content bodies remain solely in their content records.
CREATE TABLE spaces_edit_receipts (
  space_id TEXT NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
  mutation_id TEXT NOT NULL,
  request_hash TEXT NOT NULL,
  applied_revision INTEGER NOT NULL,
  changes TEXT NOT NULL CHECK (json_valid(changes)),
  selection_before TEXT NOT NULL CHECK (json_valid(selection_before)),
  selection_after TEXT NOT NULL CHECK (json_valid(selection_after)),
  invalidated_at INTEGER,
  created_at INTEGER NOT NULL,
  PRIMARY KEY (space_id,mutation_id)
);
CREATE INDEX spaces_edit_receipts_created ON spaces_edit_receipts(space_id,created_at);
-- Deleted snapshots are lifecycle records, never an alternate active structure.
CREATE TABLE spaces_editor_trash (
  space_id TEXT NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
  item_id TEXT NOT NULL,
  batch_id TEXT NOT NULL,
  structure TEXT NOT NULL CHECK (json_valid(structure)),
  layout TEXT NOT NULL CHECK (json_valid(layout)),
  world_transform TEXT NOT NULL CHECK (json_valid(world_transform)),
  deleted_at INTEGER NOT NULL,
  PRIMARY KEY (space_id,item_id)
);
CREATE INDEX spaces_editor_trash_batch ON spaces_editor_trash(space_id,batch_id,deleted_at);
CREATE TABLE spaces_deleted_relations (
  space_id TEXT NOT NULL REFERENCES spaces(id) ON DELETE CASCADE,
  relation_id TEXT PRIMARY KEY NOT NULL REFERENCES spaces_relations(id) ON DELETE CASCADE,
  batch_id TEXT NOT NULL,
  style TEXT CHECK (style IS NULL OR json_valid(style)),
  deleted_at INTEGER NOT NULL
);
'''
write(name,schema)
# Validate only a fresh, transient installation using the current checked-in schema sequence.
probe=sqlite3.connect(':memory:')
for migration in sorted((ROOT/'src-tauri/migrations/spaces').glob('*.sql')):
    probe.executescript(files.get(str(migration),migration.read_text()))
if probe.execute('PRAGMA foreign_key_check').fetchall(): raise RuntimeError('Fresh schema has invalid foreign keys')

write('src-tauri/src/spaces/canvas_state/stored_document.rs', '''
use serde::{Deserialize, Serialize};
use super::contract::{SpaceEditorDocument, SpaceStructure, StructureLayoutDocument, CanvasRelationStyleV1, CanvasViewport};
/** The camera is assembled at read boundaries and is never serialized into edit storage. */
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct StoredEditorDocument {
    version: u8,
    structure: SpaceStructure,
    layout: StructureLayoutDocument,
    #[serde(skip_serializing_if = "Option::is_none")] relation_styles: Option<Vec<CanvasRelationStyleV1>>,
}
impl StoredEditorDocument {
    pub fn from_document(document: &SpaceEditorDocument) -> Self {
        Self { version: document.version, structure: document.structure.clone(), layout: document.layout.clone(), relation_styles: document.relation_styles.clone() }
    }
    pub fn into_document(self, viewport: CanvasViewport) -> SpaceEditorDocument {
        SpaceEditorDocument { version: self.version, structure: self.structure, layout: self.layout, relation_styles: self.relation_styles, viewport }
    }
}
''')
write('src-tauri/src/spaces/canvas_state/viewport.rs', '''
use serde::Deserialize;
use super::contract::CanvasViewport;
use crate::spaces::db;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SaveViewportRequest { pub space_id: String, pub viewport: CanvasViewport }
pub(in crate::spaces) async fn read(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str) -> Result<CanvasViewport, String> {
    let value: Option<String> = sqlx::query_scalar("SELECT viewport FROM spaces_canvas_viewports WHERE space_id=$1").bind(space_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?;
    let viewport = match value { Some(value) => serde_json::from_str(&value).map_err(db::json_error)?, None => CanvasViewport { x: 0.0, y: 0.0, zoom: 1.0 } };
    validate(&viewport)?; Ok(viewport)
}
pub(in crate::spaces) async fn save_in_transaction(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, viewport: &CanvasViewport) -> Result<(), String> {
    validate(viewport)?;
    sqlx::query("INSERT INTO spaces_canvas_viewports(space_id,viewport,updated_at) VALUES($1,$2,$3) ON CONFLICT(space_id) DO UPDATE SET viewport=excluded.viewport,updated_at=excluded.updated_at")
        .bind(space_id).bind(serde_json::to_string(viewport).map_err(db::json_error)?).bind(crate::clock::unix_time_millis()?)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}
pub(super) async fn save(pool: &sqlx::SqlitePool, request: SaveViewportRequest) -> Result<(), String> {
    let mut tx = db::begin_write(pool).await?;
    save_in_transaction(&mut tx, &request.space_id, &request.viewport).await?;
    tx.commit().await.map_err(db::sql_error)
}
fn validate(value: &CanvasViewport) -> Result<(), String> {
    if !value.x.is_finite() || !value.y.is_finite() || !value.zoom.is_finite() || value.zoom <= 0.0 { return Err("Invalid camera state".into()); } Ok(())
}
''')
write('src-tauri/src/spaces/canvas_state/repository.rs', '''
use sqlx::Row;
use crate::spaces::db;
use super::{contract::{SpaceEditorDocument, CanvasViewport}, stored_document::StoredEditorDocument};
pub(super) async fn load(pool: &sqlx::SqlitePool, space_id: &str) -> Result<Option<(SpaceEditorDocument, i64)>, String> {
    let mut tx = pool.begin().await.map_err(db::sql_error)?;
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spaces WHERE id=$1)").bind(space_id).fetch_one(&mut *tx).await.map_err(db::sql_error)?;
    if !exists { tx.commit().await.map_err(db::sql_error)?; return Ok(None); }
    let state = load_in_transaction(&mut tx, space_id).await?;
    super::validation::validate_content_references(&mut tx, space_id, &state.0).await?;
    tx.commit().await.map_err(db::sql_error)?; Ok(Some(state))
}
pub(in crate::spaces) async fn load_in_transaction(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str) -> Result<(SpaceEditorDocument, i64), String> {
    let row = sqlx::query("SELECT document,revision FROM spaces_canvas_state WHERE space_id=$1").bind(space_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?;
    let viewport = super::viewport::read(tx, space_id).await?;
    match row { Some(row) => decode(row, viewport), None => { let mut empty = SpaceEditorDocument::empty(); empty.viewport = viewport; Ok((empty,0)) } }
}
pub(super) async fn save(pool: &sqlx::SqlitePool, space_id: &str, base_revision: i64, document: &SpaceEditorDocument) -> Result<i64, String> {
    let mut tx = db::begin_write(pool).await?;
    let revision = save_in_transaction(&mut tx, space_id, base_revision, document).await?;
    tx.commit().await.map_err(db::sql_error)?; Ok(revision)
}
pub(in crate::spaces) async fn save_in_transaction(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, base_revision: i64, document: &SpaceEditorDocument) -> Result<i64, String> {
    if !(0..9_007_199_254_740_991).contains(&base_revision) { return Err("Invalid edit revision".into()); }
    super::validate_document(document)?;
    super::validation::validate_content_references(tx, space_id, document).await?;
    let revision = base_revision + 1;
    let encoded = serde_json::to_string(&StoredEditorDocument::from_document(document)).map_err(db::json_error)?;
    let now = crate::clock::unix_time_millis()?;
    let result = if base_revision == 0 {
        sqlx::query("INSERT INTO spaces_canvas_state(space_id,document,revision,updated_at) SELECT $1,$2,$3,$4 WHERE NOT EXISTS(SELECT 1 FROM spaces_canvas_state WHERE space_id=$1)")
            .bind(space_id).bind(encoded).bind(revision).bind(now).execute(&mut **tx).await
    } else {
        sqlx::query("UPDATE spaces_canvas_state SET document=$1,revision=$2,updated_at=$3 WHERE space_id=$4 AND revision=$5")
            .bind(encoded).bind(revision).bind(now).bind(space_id).bind(base_revision).execute(&mut **tx).await
    }.map_err(db::sql_error)?;
    if result.rows_affected() != 1 { return Err("Space edit revision conflict".into()); }
    crate::spaces::assets::synchronize_canvas_references(tx, space_id, document).await?;
    Ok(revision)
}
fn decode(row: sqlx::sqlite::SqliteRow, viewport: CanvasViewport) -> Result<(SpaceEditorDocument, i64), String> {
    let encoded: String = row.try_get("document").map_err(db::sql_error)?;
    let document = serde_json::from_str::<StoredEditorDocument>(&encoded).map_err(db::json_error)?.into_document(viewport);
    super::validate_document(&document)?;
    let revision: i64 = row.try_get("revision").map_err(db::sql_error)?;
    if !(1..=9_007_199_254_740_991).contains(&revision) { return Err("Invalid stored edit revision".into()); }
    Ok((document,revision))
}
''')
write('src-tauri/src/spaces/assets/reclamation/canvas_references.rs', '''
use crate::spaces::{canvas_state::SpaceEditorDocument, db};
pub(in crate::spaces) async fn synchronize(tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>, space_id: &str, document: &SpaceEditorDocument) -> Result<(), String> {
    sqlx::query("DELETE FROM spaces_canvas_asset_references WHERE space_id=$1").bind(space_id).execute(&mut **tx).await.map_err(db::sql_error)?;
    for layout in &document.layout.items {
        let Some(path) = layout.appearance().and_then(|value| value.background.as_deref()) else { continue; };
        if path.trim().is_empty() { return Err("A background resource path cannot be empty".into()); }
        sqlx::query("INSERT INTO spaces_canvas_asset_references(space_id,item_id,path) VALUES($1,$2,$3)")
            .bind(space_id).bind(layout.id()).bind(path).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}
''')
name='src-tauri/src/spaces/assets/reclamation/reference_graph.rs';text=read(name)
text=text.replace('spaces_canvas_asset_references WHERE space_id=$1 AND node_id IN', 'spaces_canvas_asset_references WHERE space_id=$1 AND item_id IN')
write(name,text)
name='src-tauri/src/spaces/canvas_state/mod.rs';text=read(name)
text=text.replace('mod repository;', 'mod repository;\nmod stored_document;\npub(in crate::spaces) mod viewport;')
text+='''
#[tauri::command]
pub(crate) async fn save_editor_viewport<R: Runtime>(app: AppHandle<R>, request: viewport::SaveViewportRequest) -> Result<(), String> {
    let pool = db::pool(&app).await?; viewport::save(pool, request).await
}
'''
write(name,text)
name='src-tauri/src/spaces/mod.rs';write(name,read(name).replace('            crate::spaces::canvas_state::save_canvas_state,','            crate::spaces::canvas_state::save_canvas_state,\n            crate::spaces::canvas_state::save_editor_viewport,'))
name='src-tauri/build.rs';write(name,read(name).replace('                        "save_canvas_state",','                        "save_canvas_state",\n                        "save_editor_viewport",'))
name='src-tauri/capabilities/spaces.json'
if (ROOT/name).exists(): write(name,read(name).replace('"spaces:allow-save-canvas-state",','"spaces:allow-save-canvas-state",\n    "spaces:allow-save-editor-viewport",'))
write('src-tauri/src/spaces/editing/changes.rs', '''
use std::collections::{BTreeMap, HashSet};
use serde::{Deserialize, Serialize};
use crate::spaces::canvas_state::contract::{SpaceEditorDocument, StructureItem, StructureLayout, CanvasRelationStyleV1};
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::spaces) struct RecordChange<T> { pub id: String, pub before: Option<T>, pub after: Option<T> }
#[derive(Clone, Debug, Deserialize, Serialize)]
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
''')
write('src-tauri/src/spaces/editing/mod.rs','''pub(in crate::spaces) mod changes;
''')
name='src-tauri/src/spaces/mod.rs';text=read(name)
if 'mod editing;' not in text: text='pub(in crate::spaces) mod editing;\n'+text
write(name,text)
for name,text in files.items():
    path=ROOT/name;path.parent.mkdir(parents=True,exist_ok=True)
    if not path.exists() or path.read_text()!=text:path.write_text(text)
Path(__file__).write_text('"""No pending source edits. The branch is verified as committed."""\nprint("No pending source edits")\n')
print('Added independent camera and exact edit-receipt storage; verified the fresh schema in memory.')
