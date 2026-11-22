use std::collections::{HashMap, HashSet};

use serde_json::{Map, Value};
use sqlx::{Sqlite, Transaction};

use super::state::{resolve_node, ApplyState};
use super::CommitContext;
use crate::spaces::derivation::placement::{
    self as derivation_placement, DerivedOutputs, SetMember,
};
use crate::spaces::knowledge::change_commit::{
    contracts::KnowledgeChange,
    mutations::{self, ObjectInsert, ObjectUpdate},
};
use crate::spaces::knowledge::{events, grounding::ResolvedRunCitation};
use crate::spaces::{nodes::contracts::InsertNode, structure};

const OBJECT_WIDTH: f64 = 256.0;
const OBJECT_HEIGHT: f64 = 176.0;

pub(super) async fn create(
    tx: &mut Transaction<'_, Sqlite>,
    context: CommitContext<'_>,
    operations: &[KnowledgeChange],
    placement: &mut structure::repository::SpaceStructureSnapshot,
    state: &mut ApplyState,
) -> Result<(), String> {
    let CommitContext {
        space_id,
        actor,
        now,
    } = context;
    let planned_positions = plan_auto_positions(
        tx,
        space_id,
        operations,
        &state.citation_by_cite_id,
        placement,
    )
    .await?;
    for operation in operations {
        let KnowledgeChange::CreateObject {
            temp_id,
            object_type_id,
            title,
            summary,
            status,
            properties,
            position,
        } = operation
        else {
            continue;
        };
        let id = uuid::Uuid::new_v4().to_string();
        let mut data = build_object_content(
            object_type_id,
            title,
            summary.as_deref(),
            status,
            properties.clone(),
        );
        crate::spaces::knowledge::model::revision::apply(&mut data)?;
        let object_type_id =
            crate::spaces::knowledge::validation::validate_node(tx, space_id, "object", &data)
                .await?
                .ok_or_else(|| "Knowledge object type validation failed".to_string())?;
        let (initial_x, initial_y) = match position {
            Some(position) => (position.x, position.y),
            None => match planned_positions.get(temp_id.as_str()) {
                Some(&(x, y)) => (x, y),
                None => structure::place_new_content(
                    &placement.structure,
                    &placement.layouts,
                    OBJECT_WIDTH,
                    OBJECT_HEIGHT,
                )?,
            },
        };
        let node = InsertNode {
            id: id.clone(),
            kind: "object".into(),
            initial_x,
            initial_y,
            width: OBJECT_WIDTH,
            height: OBJECT_HEIGHT,
            sizing_mode: structure::SizingMode::Frame,
            appearance: None,
            file_subpath: None,
            data,
            source_chunks: Vec::new(),
        };
        mutations::insert_object(
            tx,
            ObjectInsert {
                space_id,
                id: &id,
                object_type_id: &object_type_id,
                initial_x,
                initial_y,
                data: &node.data,
                now,
            },
        )
        .await?;
        emit_created(tx, space_id, actor, temp_id, &id, &node.data, now).await?;
        structure::creation::append_content(placement, std::slice::from_ref(&node), None)?;
        state.node_id_by_temp_id.insert(temp_id.clone(), id.clone());
        state.resolved_id_by_temp_id.insert(temp_id.clone(), id);
        state.created_nodes.push(node);
    }
    Ok(())
}

/// Clusters newly extracted knowledge objects beside their linked canvas source nodes using
/// the shared derivation placement solver instead of scattering them across empty canvas space.
async fn plan_auto_positions(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    operations: &[KnowledgeChange],
    citation_by_cite_id: &HashMap<String, ResolvedRunCitation>,
    placement: &structure::repository::SpaceStructureSnapshot,
) -> Result<HashMap<String, (f64, f64)>, String> {
    let temp_nodes: HashSet<&str> = operations
        .iter()
        .filter_map(|op| match op {
            KnowledgeChange::CreateObject { temp_id, .. } => Some(temp_id.as_str()),
            _ => None,
        })
        .collect();
    let auto_temps: Vec<&str> = operations
        .iter()
        .filter_map(|op| match op {
            KnowledgeChange::CreateObject {
                temp_id,
                position: None,
                ..
            } => Some(temp_id.as_str()),
            _ => None,
        })
        .collect();
    if auto_temps.is_empty() {
        return Ok(HashMap::new());
    }
    let mut source_ids = Vec::new();
    let mut push_source = |candidate: &str| {
        if !temp_nodes.contains(candidate) && !source_ids.iter().any(|id| id == candidate) {
            source_ids.push(candidate.to_owned());
        }
    };
    for operation in operations {
        match operation {
            KnowledgeChange::CreateRelation {
                source_id,
                target_id,
                ..
            } => {
                if temp_nodes.contains(source_id.as_str()) {
                    push_source(target_id);
                }
                if temp_nodes.contains(target_id.as_str()) {
                    push_source(source_id);
                }
            }
            KnowledgeChange::AddProvenance { source_node_id, .. } => {
                push_source(source_node_id);
            }
            KnowledgeChange::AddCitedProvenance { cite_id, .. } => {
                if let Some(resolved) = citation_by_cite_id.get(cite_id.as_str()) {
                    push_source(&resolved.source_node_id);
                }
            }
            _ => {}
        }
    }
    if source_ids.is_empty() {
        return Ok(HashMap::new());
    }
    let size = structure::Size {
        width: OBJECT_WIDTH,
        height: OBJECT_HEIGHT,
    };
    let empty_data = Value::Object(Map::new());
    let members = auto_temps
        .iter()
        .map(|_| SetMember::new(&empty_data, &size, structure::SizingMode::Frame))
        .collect();
    let cells = derivation_placement::plan(
        tx,
        space_id,
        &placement.structure,
        &placement.layouts,
        DerivedOutputs {
            sources: &source_ids,
            reserved: &[],
            members,
        },
    )
    .await?;
    Ok(auto_temps
        .into_iter()
        .zip(cells)
        .map(|(temp_id, cell)| (temp_id.to_owned(), (cell.x, cell.y)))
        .collect())
}

pub(super) async fn update(
    tx: &mut Transaction<'_, Sqlite>,
    context: CommitContext<'_>,
    operations: &[KnowledgeChange],
    state: &ApplyState,
) -> Result<(), String> {
    let CommitContext {
        space_id,
        actor,
        now,
    } = context;
    for operation in operations {
        let KnowledgeChange::UpdateObject {
            node_id,
            title,
            summary,
            status,
            properties,
        } = operation
        else {
            continue;
        };
        let id = resolve_node(tx, space_id, &state.node_id_by_temp_id, node_id).await?;
        mutations::update_object(
            tx,
            ObjectUpdate {
                space_id,
                id: &id,
                title,
                summary,
                status,
                properties,
                actor,
                now,
            },
        )
        .await?;
    }
    Ok(())
}

async fn emit_created(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    actor: events::EventActor<'_>,
    temp_id: &str,
    node_id: &str,
    node_payload: &Value,
    now: i64,
) -> Result<(), String> {
    let payload = serde_json::json!({"changeTempId": temp_id});
    events::append(
        tx,
        events::KnowledgeEvent::new(
            space_id,
            events::EventEntity::new("node", node_id),
            "created",
            actor,
            &payload,
            now,
        ),
    )
    .await?;
    let before = serde_json::json!({});
    events::append_object_status_transition(
        tx,
        space_id,
        node_id,
        &before,
        node_payload,
        actor,
        now,
    )
    .await
}

fn build_object_content(
    object_type_id: &str,
    title: &str,
    summary: Option<&str>,
    status: &str,
    properties: Map<String, Value>,
) -> Value {
    let mut data = serde_json::json!({
        "kind": "object",
        "objectTypeId": object_type_id,
        "title": title,
        "status": status,
        "properties": properties,
    });
    if let Some(summary) = summary {
        data.as_object_mut()
            .expect("object data")
            .insert("summary".into(), Value::String(summary.into()));
    }
    data
}
