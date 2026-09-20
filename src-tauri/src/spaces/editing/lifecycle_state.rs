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
