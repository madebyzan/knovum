use sqlx::{Row, Sqlite, Transaction};

use super::state::{resolve_node, resolve_relation, ApplyState};
use super::CommitContext;
use crate::spaces::knowledge::change_commit::{
    contracts::KnowledgeChange,
    mutations::{self, ProvenanceInsert},
};
use crate::spaces::{db, knowledge::events};

pub(super) async fn create_relations(
    tx: &mut Transaction<'_, Sqlite>,
    context: CommitContext<'_>,
    operations: &[KnowledgeChange],
    state: &mut ApplyState,
) -> Result<(), String> {
    let CommitContext {
        space_id,
        actor,
        now,
    } = context;
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
        let source = resolve_node(tx, space_id, &state.node_id_by_temp_id, source_id).await?;
        let target = resolve_node(tx, space_id, &state.node_id_by_temp_id, target_id).await?;
        let (source, target) =
            crate::spaces::knowledge::relations::canonical_endpoints(kind, &source, &target);
        crate::spaces::knowledge::relations::validate_endpoints(tx, space_id, &source, &target)
            .await?;
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO spaces_relations(id,space_id,source_id,target_id,kind,rationale,status,confidence,actor_kind,actor_id,created_at,updated_at) \
             VALUES($1,$2,$3,$4,$5,$6,'active',$7,$8,$9,$10,$10)",
        )
        .bind(&id)
        .bind(space_id)
        .bind(source)
        .bind(target)
        .bind(kind)
        .bind(rationale)
        .bind(confidence)
        .bind(actor.kind())
        .bind(actor.id())
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        emit_relation_created(tx, space_id, actor, &id, kind, now).await?;
        state
            .relation_id_by_temp_id
            .insert(temp_id.clone(), id.clone());
        state.resolved_id_by_temp_id.insert(temp_id.clone(), id);
    }
    Ok(())
}

async fn emit_relation_created(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    actor: events::EventActor<'_>,
    relation_id: &str,
    kind: &str,
    now: i64,
) -> Result<(), String> {
    let payload = serde_json::json!({"kind": kind});
    events::append(
        tx,
        events::KnowledgeEvent::new(
            space_id,
            events::EventEntity::new("relation", relation_id),
            "created",
            actor,
            &payload,
            now,
        ),
    )
    .await
}

pub(super) async fn update_relations(
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
        let KnowledgeChange::UpdateRelation {
            relation_id,
            status,
            rationale,
            confidence,
        } = operation
        else {
            continue;
        };
        let id = resolve_relation(tx, space_id, &state.relation_id_by_temp_id, relation_id).await?;
        let current = sqlx::query(
            "SELECT status, rationale, confidence FROM spaces_relations \
             WHERE space_id=$1 AND id=$2",
        )
        .bind(space_id)
        .bind(&id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(db::sql_error)?
        .ok_or("Knowledge Change Set relation does not exist in this Space")?;
        let next_status = status
            .clone()
            .unwrap_or_else(|| current.get::<String, _>("status"));
        let next_rationale = rationale
            .clone()
            .unwrap_or_else(|| current.get::<String, _>("rationale"));
        let next_confidence = confidence.or_else(|| current.get::<Option<f64>, _>("confidence"));
        sqlx::query(
            "UPDATE spaces_relations SET status=$1, rationale=$2, confidence=$3, updated_at=$4 \
             WHERE space_id=$5 AND id=$6",
        )
        .bind(&next_status)
        .bind(&next_rationale)
        .bind(next_confidence)
        .bind(now)
        .bind(space_id)
        .bind(&id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        let payload = serde_json::json!({ "status": next_status });
        events::append(
            tx,
            events::KnowledgeEvent::new(
                space_id,
                events::EventEntity::new("relation", &id),
                "updated",
                actor,
                &payload,
                now,
            ),
        )
        .await?;
    }
    Ok(())
}

pub(super) async fn add_provenance(
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
        match operation {
            KnowledgeChange::AddProvenance {
                target_kind,
                target_id,
                source_node_id,
                locator,
                source_revision,
                quote,
            } => {
                let source =
                    resolve_node(tx, space_id, &state.node_id_by_temp_id, source_node_id).await?;
                let target =
                    resolve_provenance_target(tx, space_id, state, target_kind, target_id).await?;
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
                        actor,
                        now,
                    },
                )
                .await?;
            }
            KnowledgeChange::AddCitedProvenance {
                target_kind,
                target_id,
                cite_id,
            } => {
                let resolved =
                    state
                        .citation_by_cite_id
                        .get(cite_id.as_str())
                        .ok_or_else(|| {
                            format!("citation-unavailable: citeId {cite_id} was not resolved")
                        })?;
                let target =
                    resolve_provenance_target(tx, space_id, state, target_kind, target_id).await?;
                mutations::insert_provenance(
                    tx,
                    ProvenanceInsert {
                        space_id,
                        target_kind,
                        target_id: &target,
                        source_node_id: &resolved.source_node_id,
                        locator: &resolved.locator,
                        source_revision: &resolved.source_revision,
                        quote: &resolved.quote,
                        actor,
                        now,
                    },
                )
                .await?;
            }
            _ => {}
        }
    }
    Ok(())
}

async fn resolve_provenance_target(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    state: &ApplyState,
    target_kind: &str,
    target_id: &str,
) -> Result<String, String> {
    if target_kind == "relation" {
        resolve_relation(tx, space_id, &state.relation_id_by_temp_id, target_id).await
    } else {
        resolve_node(tx, space_id, &state.node_id_by_temp_id, target_id).await
    }
}
