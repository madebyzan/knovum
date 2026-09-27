use serde_json::{Map, Value};
use sqlx::{Sqlite, Transaction};

use super::state::{resolve_node, ApplyState};
use crate::spaces::knowledge::change_commit::{
    contract::KnowledgeChange,
    mutations::{self, ObjectInsert, ObjectUpdate},
};
use crate::spaces::knowledge::{contracts, events};

pub(super) async fn create(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    agent_id: &str,
    operations: &[KnowledgeChange],
    now: i64,
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
        let mut data = object_data(
            object_type_id,
            title,
            summary.as_deref(),
            status,
            properties.clone(),
        );
        crate::spaces::knowledge::indexing::revision::apply(&mut data)?;
        let object_type_id = contracts::validate_node(tx, space_id, &id, "object", &data)
            .await?
            .ok_or_else(|| "Knowledge object type validation failed".to_string())?;
        let (initial_x, initial_y) = object_position(position.as_ref(), state.create_index);
        state.create_index += 1;
        mutations::insert_object(
            tx,
            ObjectInsert {
                space_id,
                id: &id,
                object_type_id: &object_type_id,
                initial_x,
                initial_y,
                data: &data,
                now,
            },
        )
        .await?;
        emit_created(tx, space_id, agent_id, temp_id, &id, &data, now).await?;
        state.node_ids.insert(temp_id.clone(), id.clone());
        state.resolved_ids.insert(temp_id.clone(), id);
    }
    Ok(())
}

pub(super) async fn update(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    agent_id: &str,
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
        let id = resolve_node(tx, space_id, &state.node_ids, node_id).await?;
        mutations::update_object(
            tx,
            ObjectUpdate {
                space_id,
                id: &id,
                title,
                summary,
                status,
                properties,
                agent_id,
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
    agent_id: &str,
    temp_id: &str,
    id: &str,
    data: &Value,
    now: i64,
) -> Result<(), String> {
    let payload = serde_json::json!({"changeTempId": temp_id});
    events::append(
        tx,
        events::KnowledgeEvent::new(
            space_id,
            events::EventEntity::new("node", id),
            "created",
            events::EventActor::new("agent", Some(agent_id)),
            &payload,
            now,
        ),
    )
    .await?;
    let before = serde_json::json!({});
    events::append_object_status_transition(
        tx,
        space_id,
        id,
        &before,
        data,
        events::EventActor::new("agent", Some(agent_id)),
        now,
    )
    .await
}

fn object_data(
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

fn object_position(
    position: Option<&crate::spaces::knowledge::change_commit::contract::Position>,
    create_index: usize,
) -> (f64, f64) {
    position.map_or_else(
        || {
            (
                96.0 + (create_index % 3) as f64 * 300.0,
                84.0 + (create_index / 3) as f64 * 180.0,
            )
        },
        |value| (value.x, value.y),
    )
}
