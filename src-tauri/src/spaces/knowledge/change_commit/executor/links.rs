use sqlx::{Sqlite, Transaction};

use super::state::{resolve_node, resolve_relation, ApplyState};
use crate::spaces::knowledge::change_commit::{
    contract::KnowledgeChange,
    mutations::{self, ProvenanceInsert},
};
use crate::spaces::{
    db,
    knowledge::{contracts, events},
};

pub(super) async fn create_relations(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    agent_id: &str,
    operations: &[KnowledgeChange],
    now: i64,
    state: &mut ApplyState,
) -> Result<(), String> {
    for operation in operations {
        let KnowledgeChange::CreateRelation {
            temp_id,
            source_id,
            target_id,
            kind,
            rationale,
            confidence,
        } = operation
        else {
            continue;
        };
        let source = resolve_node(tx, space_id, &state.node_ids, source_id).await?;
        let target = resolve_node(tx, space_id, &state.node_ids, target_id).await?;
        let (source, target) = contracts::canonical_relation_endpoints(kind, &source, &target);
        contracts::validate_relation_endpoints(tx, space_id, &source, &target).await?;
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO spaces_relations(id,space_id,source_id,target_id,kind,rationale,status,confidence,actor_kind,actor_id,created_at,updated_at) \
             VALUES($1,$2,$3,$4,$5,$6,'active',$7,'agent',$8,$9,$9)",
        )
        .bind(&id)
        .bind(space_id)
        .bind(source)
        .bind(target)
        .bind(kind)
        .bind(rationale)
        .bind(confidence)
        .bind(agent_id)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        emit_relation_created(tx, space_id, agent_id, &id, kind, now).await?;
        state.relation_ids.insert(temp_id.clone(), id.clone());
        state.resolved_ids.insert(temp_id.clone(), id);
    }
    Ok(())
}

async fn emit_relation_created(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    agent_id: &str,
    id: &str,
    kind: &str,
    now: i64,
) -> Result<(), String> {
    let payload = serde_json::json!({"kind": kind});
    events::append(
        tx,
        events::KnowledgeEvent::new(
            space_id,
            events::EventEntity::new("relation", id),
            "created",
            events::EventActor::new("agent", Some(agent_id)),
            &payload,
            now,
        ),
    )
    .await
}

pub(super) async fn add_provenance(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    agent_id: &str,
    operations: &[KnowledgeChange],
    now: i64,
    state: &ApplyState,
) -> Result<(), String> {
    for operation in operations {
        let KnowledgeChange::AddProvenance {
            target_kind,
            target_id,
            source_node_id,
            locator,
            source_revision,
            quote,
        } = operation
        else {
            continue;
        };
        let source = resolve_node(tx, space_id, &state.node_ids, source_node_id).await?;
        let target = if target_kind == "relation" {
            resolve_relation(tx, space_id, &state.relation_ids, target_id).await?
        } else {
            resolve_node(tx, space_id, &state.node_ids, target_id).await?
        };
        mutations::insert_provenance(
            tx,
            ProvenanceInsert {
                space_id,
                target_kind,
                target_id: &target,
                source_node_id: &source,
                locator,
                source_revision,
                quote,
                agent_id,
                now,
            },
        )
        .await?;
    }
    Ok(())
}
