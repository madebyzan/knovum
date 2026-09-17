use super::{contracts, locator, semantic_nodes, ProvenanceInput};
use crate::spaces::knowledge::relations;

pub(super) async fn ensure_target(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    input: &ProvenanceInput,
) -> Result<(), String> {
    contracts::validate_provenance_node(
        tx,
        &input.space_id,
        &input.source_node_id,
        "Provenance source",
    )
    .await?;
    if input.target_kind == "node" {
        return semantic_nodes::require(tx, &input.space_id, &input.target_id, "Provenance target")
            .await;
    }
    if !relations::exists(tx, &input.space_id, &input.target_id).await? {
        return Err("Provenance target does not exist in this Space".to_string());
    }
    Ok(())
}

pub(super) fn validate_input(input: &ProvenanceInput) -> Result<(), String> {
    contracts::validate_provenance_target_kind(&input.target_kind)?;
    if input.source_revision.trim().is_empty() {
        return Err("Provenance source revision is required".into());
    }
    locator::validate(&input.locator, Some(&input.source_revision))?;
    contracts::validate_provenance_quote(&input.quote)
}
