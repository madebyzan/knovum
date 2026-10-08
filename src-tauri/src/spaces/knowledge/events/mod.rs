use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use crate::spaces::db;

pub(crate) mod validation;

pub struct EventEntity<'a> {
    kind: &'a str,
    id: &'a str,
}

impl<'a> EventEntity<'a> {
    pub fn new(kind: &'a str, id: &'a str) -> Self {
        Self { kind, id }
    }
}

#[derive(Clone, Copy)]
pub struct EventActor<'a> {
    kind: &'a str,
    id: Option<&'a str>,
}

impl<'a> EventActor<'a> {
    pub fn new(kind: &'a str, id: Option<&'a str>) -> Self {
        Self { kind, id }
    }

    pub fn kind(self) -> &'a str {
        self.kind
    }

    pub fn id(self) -> Option<&'a str> {
        self.id
    }
}

pub struct KnowledgeEvent<'a> {
    space_id: &'a str,
    entity: EventEntity<'a>,
    event_kind: &'a str,
    actor: EventActor<'a>,
    payload: &'a Value,
    now: i64,
}

impl<'a> KnowledgeEvent<'a> {
    pub fn new(
        space_id: &'a str,
        entity: EventEntity<'a>,
        event_kind: &'a str,
        actor: EventActor<'a>,
        payload: &'a Value,
        now: i64,
    ) -> Self {
        Self {
            space_id,
            entity,
            event_kind,
            actor,
            payload,
            now,
        }
    }
}

pub async fn append(
    tx: &mut Transaction<'_, Sqlite>,
    event: KnowledgeEvent<'_>,
) -> Result<(), String> {
    validation::validate_pair(event.entity.kind, event.event_kind)?;
    validation::validate_actor_kind(event.actor.kind)?;
    let payload = serde_json::to_string(event.payload).map_err(db::json_error)?;
    sqlx::query(
        "INSERT INTO spaces_knowledge_events(\
           id, space_id, entity_kind, entity_id, event_kind, actor_kind, actor_id, payload, created_at\
         ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(event.space_id)
    .bind(event.entity.kind)
    .bind(event.entity.id)
    .bind(event.event_kind)
    .bind(event.actor.kind)
    .bind(event.actor.id)
    .bind(payload)
    .bind(event.now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}

pub async fn append_object_status_transition(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
    before: &Value,
    after: &Value,
    actor: EventActor<'_>,
    now: i64,
) -> Result<(), String> {
    let before_status = before.get("status").and_then(Value::as_str);
    let after_status = after.get("status").and_then(Value::as_str);
    if before_status == after_status {
        return Ok(());
    }
    let Some(after_status) = after_status else {
        return Ok(());
    };
    let event_kind = match after_status {
        "draft" => "drafted",
        "accepted" => "accepted",
        "contested" => "contested",
        "superseded" => "superseded",
        "invalidated" => "invalidated",
        "open" => "opened",
        "resolved" => "resolved",
        _ => return Err("Unsupported knowledge object status transition".into()),
    };
    let payload = serde_json::json!({
        "beforeStatus": before_status,
        "afterStatus": after_status,
    });
    append(
        tx,
        KnowledgeEvent::new(
            space_id,
            EventEntity::new("node", node_id),
            event_kind,
            actor,
            &payload,
            now,
        ),
    )
    .await
}
