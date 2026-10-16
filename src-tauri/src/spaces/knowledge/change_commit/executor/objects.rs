use serde_json::{Map, Value};
use sqlx::{Sqlite, Transaction};

use super::state::{resolve_node, ApplyState};
use crate::spaces::knowledge::change_commit::{
    contracts::KnowledgeChange,
    mutations::{self, ObjectInsert, ObjectUpdate},
};
use crate::spaces::knowledge::events;
use crate::spaces::{nodes::contracts::InsertNode, structure};

const OBJECT_WIDTH: f64 = 256.0;
const OBJECT_HEIGHT: f64 = 176.0;

pub(super) async fn create(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    actor: events::EventActor<'_>,
    operations: &[KnowledgeChange],
    now: i64,
    placement: &mut structure::repository::SpaceStructureSnapshot,
    state: &mut ApplyState,
) -> Result<(), String> {
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
            None => structure::place_new_content(
                &placement.structure,
                &placement.layouts,
                &[],
                &[],
                OBJECT_WIDTH,
                OBJECT_HEIGHT,
            )?,
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

pub(super) async fn update(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    actor: events::EventActor<'_>,
    operations: &[KnowledgeChange],
    now: i64,
    state: &ApplyState,
) -> Result<(), String> {
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
