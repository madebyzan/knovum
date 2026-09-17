use sqlx::{Sqlite, Transaction};

mod validation;

use validation::{validate_event_entity, validate_provenance_target, validate_relation_lifecycle};

use super::contracts::RestorePackageRequest;
use crate::spaces::{
    db,
    knowledge::{contracts, locator},
};

pub(super) async fn insert_relations(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.relations {
        contracts::validate_relation_kind(&row.kind)?;
        contracts::validate_relation_status(&row.status)?;
        contracts::validate_relation_rationale(&row.rationale)?;
        validate_relation_lifecycle(&row.status, row.status_before_invalidation.as_deref())?;
        contracts::validate_actor_kind(&row.actor_kind)?;
        contracts::validate_confidence(row.confidence)?;
        let (source_id, target_id) =
            contracts::canonical_relation_endpoints(&row.kind, &row.source_id, &row.target_id);
        contracts::validate_relation_endpoints(tx, &request.space.id, &source_id, &target_id)
            .await?;
        sqlx::query(
            "INSERT INTO spaces_relations(id,space_id,source_id,target_id,kind,rationale,status,status_before_invalidation,confidence,actor_kind,actor_id,created_at,updated_at) \
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)",
        ).bind(&row.id).bind(&request.space.id).bind(&source_id).bind(&target_id)
            .bind(&row.kind).bind(&row.rationale).bind(&row.status).bind(&row.status_before_invalidation).bind(row.confidence)
            .bind(&row.actor_kind).bind(&row.actor_id).bind(row.created_at).bind(row.updated_at)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

pub(super) async fn insert_provenance(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.provenance {
        contracts::validate_provenance_target_kind(&row.target_kind)?;
        contracts::validate_provenance_status(&row.status)?;
        contracts::validate_actor_kind(&row.actor_kind)?;
        if row.source_revision.trim().is_empty() {
            return Err("Provenance source revision is required".into());
        }
        locator::validate(&row.locator, Some(&row.source_revision))?;
        contracts::validate_provenance_quote(&row.quote)?;
        contracts::validate_provenance_node(
            tx,
            &request.space.id,
            &row.source_node_id,
            "Provenance source",
        )
        .await?;
        validate_provenance_target(tx, &request.space.id, &row.target_kind, &row.target_id).await?;
        let locator = serde_json::to_string(&row.locator).map_err(db::json_error)?;
        sqlx::query(
            "INSERT INTO spaces_provenance(id,space_id,target_kind,target_id,source_node_id,locator,source_revision,quote,status,actor_kind,actor_id,created_at,updated_at) \
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)",
        ).bind(&row.id).bind(&request.space.id).bind(&row.target_kind).bind(&row.target_id)
            .bind(&row.source_node_id).bind(locator).bind(&row.source_revision).bind(&row.quote)
            .bind(&row.status).bind(&row.actor_kind).bind(&row.actor_id).bind(row.created_at).bind(row.updated_at)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

pub(super) async fn insert_events(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.knowledge_events {
        contracts::validate_event_pair(&row.entity_kind, &row.event_kind)?;
        contracts::validate_actor_kind(&row.actor_kind)?;
        validate_event_entity(tx, &request.space.id, &row.entity_kind, &row.entity_id).await?;
        let payload = serde_json::to_string(&row.payload).map_err(db::json_error)?;
        sqlx::query(
            "INSERT INTO spaces_knowledge_events(id,space_id,entity_kind,entity_id,event_kind,actor_kind,actor_id,payload,created_at) \
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)",
        ).bind(&row.id).bind(&request.space.id).bind(&row.entity_kind).bind(&row.entity_id)
            .bind(&row.event_kind).bind(&row.actor_kind).bind(&row.actor_id).bind(payload).bind(row.created_at)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
