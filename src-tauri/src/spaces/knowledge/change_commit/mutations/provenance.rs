use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use crate::spaces::{
    db,
    knowledge::{contracts, events, locator, relations, semantic_nodes},
};

pub(in crate::spaces::knowledge::change_commit) struct ProvenanceInsert<'a> {
    pub space_id: &'a str,
    pub target_kind: &'a str,
    pub target_id: &'a str,
    pub source_node_id: &'a str,
    pub locator: &'a Value,
    pub source_revision: &'a str,
    pub quote: &'a str,
    pub agent_id: &'a str,
    pub now: i64,
}

pub(in crate::spaces::knowledge::change_commit) async fn insert_provenance(
    tx: &mut Transaction<'_, Sqlite>,
    input: ProvenanceInsert<'_>,
) -> Result<(), String> {
    locator::validate(input.locator, Some(input.source_revision))?;
    contracts::validate_provenance_node(
        tx,
        input.space_id,
        input.source_node_id,
        "Provenance source",
    )
    .await?;
    validate_target(tx, &input).await?;
    let locator = serde_json::to_string(input.locator).map_err(db::json_error)?;
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO spaces_provenance(id,space_id,target_kind,target_id,source_node_id,locator,source_revision,quote,status,actor_kind,actor_id,created_at,updated_at) \
         VALUES($1,$2,$3,$4,$5,$6,$7,$8,'active','agent',$9,$10,$10)",
    )
    .bind(&id)
    .bind(input.space_id)
    .bind(input.target_kind)
    .bind(input.target_id)
    .bind(input.source_node_id)
    .bind(locator)
    .bind(input.source_revision)
    .bind(input.quote)
    .bind(input.agent_id)
    .bind(input.now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    emit_created(tx, &input, &id).await
}

async fn emit_created(
    tx: &mut Transaction<'_, Sqlite>,
    input: &ProvenanceInsert<'_>,
    id: &str,
) -> Result<(), String> {
    let payload = serde_json::json!({
        "targetKind": input.target_kind,
        "targetId": input.target_id,
    });
    events::append(
        tx,
        events::KnowledgeEvent::new(
            input.space_id,
            events::EventEntity::new("provenance", id),
            "created",
            events::EventActor::new("agent", Some(input.agent_id)),
            &payload,
            input.now,
        ),
    )
    .await
}

async fn validate_target(
    tx: &mut Transaction<'_, Sqlite>,
    input: &ProvenanceInsert<'_>,
) -> Result<(), String> {
    match input.target_kind {
        "node" => {
            semantic_nodes::require(
                tx,
                input.space_id,
                input.target_id,
                "Knowledge Change Set provenance target",
            )
            .await?;
        }
        "relation" => {
            if !relations::exists(tx, input.space_id, input.target_id).await? {
                return Err(
                    "Knowledge Change Set provenance target does not exist in this Space".into(),
                );
            }
        }
        _ => return Err("Unsupported provenance target kind".into()),
    }
    Ok(())
}
