use sqlx::{Sqlite, Transaction};

mod validation;

use validation::{validate_event_entity, validate_provenance_target, validate_relation_lifecycle};

use super::contracts::RestorePackageRequest;
use crate::spaces::{db, knowledge::locator};

pub(super) async fn insert_relations(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.relations {
        crate::spaces::knowledge::relations::validate_rationale(&row.rationale)?;
        validate_relation_lifecycle(
            row.status.as_str(),
            row.status_before_invalidation.map(|status| status.as_str()),
        )?;
        crate::spaces::knowledge::relations::validate_confidence(row.confidence)?;
        let (source_id, target_id) = crate::spaces::knowledge::relations::canonical_endpoints(
            row.kind.as_str(),
            &row.source_id,
            &row.target_id,
        );
        crate::spaces::knowledge::relations::validate_endpoints(
            tx,
            &request.space.id,
            &source_id,
            &target_id,
        )
        .await?;
        sqlx::query(
            "INSERT INTO spaces_relations(id,space_id,source_id,target_id,kind,rationale,status,status_before_invalidation,confidence,actor_kind,actor_id,created_at,updated_at) \
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)",
        ).bind(&row.id).bind(&request.space.id).bind(&source_id).bind(&target_id)
            .bind(row.kind.as_str()).bind(&row.rationale).bind(row.status.as_str()).bind(row.status_before_invalidation.map(|status| status.as_str())).bind(row.confidence)
            .bind(row.actor_kind.as_str()).bind(&row.actor_id).bind(row.created_at).bind(row.updated_at)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

pub(super) async fn insert_provenance(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.provenance {
        crate::spaces::knowledge::provenance::validation::validate_target_kind(&row.target_kind)?;
        crate::spaces::knowledge::provenance::validation::validate_status(&row.status)?;
        crate::spaces::knowledge::events::validation::validate_actor_kind(&row.actor_kind)?;
        if row.source_revision.trim().is_empty() {
            return Err("Provenance source revision is required".into());
        }
        locator::validate(&row.locator, Some(&row.source_revision))?;
        crate::spaces::knowledge::provenance::validation::validate_quote(&row.quote)?;
        crate::spaces::knowledge::semantic_nodes::require(
            tx,
            &request.space.id,
            &row.source_node_id,
            "Provenance source",
        )
        .await?;
        validate_provenance_target(tx, &request.space.id, &row.target_kind, &row.target_id).await?;
        sqlx::query(
            "INSERT INTO spaces_provenance(id,space_id,target_kind,target_id,source_node_id,locator,source_revision,quote,status,actor_kind,actor_id,created_at,updated_at) \
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)",
        ).bind(&row.id).bind(&request.space.id).bind(&row.target_kind).bind(&row.target_id)
            .bind(&row.source_node_id).bind(sqlx::types::Json(&row.locator)).bind(&row.source_revision).bind(&row.quote)
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
        crate::spaces::knowledge::events::validation::validate_pair(
            &row.entity_kind,
            &row.event_kind,
        )?;
        crate::spaces::knowledge::events::validation::validate_actor_kind(&row.actor_kind)?;
        validate_event_entity(tx, &request.space.id, &row.entity_kind, &row.entity_id).await?;
        sqlx::query(
            "INSERT INTO spaces_knowledge_events(id,space_id,entity_kind,entity_id,event_kind,actor_kind,actor_id,payload,created_at) \
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)",
        ).bind(&row.id).bind(&request.space.id).bind(&row.entity_kind).bind(&row.entity_id)
            .bind(&row.event_kind).bind(&row.actor_kind).bind(&row.actor_id).bind(sqlx::types::Json(&row.payload)).bind(row.created_at)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
