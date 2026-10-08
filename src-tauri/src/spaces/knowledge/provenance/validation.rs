use super::{locator, semantic_nodes, ProvenanceInput};
use crate::spaces::knowledge::relations;

const MAX_QUOTE_CHARS: usize = 64_000;

pub(crate) fn validate_target_kind(kind: &str) -> Result<(), String> {
    match kind {
        "node" | "relation" => Ok(()),
        _ => Err("Unsupported provenance target kind".into()),
    }
}

pub(crate) fn validate_status(status: &str) -> Result<(), String> {
    match status {
        "active" | "superseded" | "invalidated" => Ok(()),
        _ => Err("Unsupported provenance status".into()),
    }
}

pub(crate) fn validate_quote(value: &str) -> Result<(), String> {
    if value.encode_utf16().count() > MAX_QUOTE_CHARS {
        return Err(format!(
            "Provenance quote exceeds the maximum of {MAX_QUOTE_CHARS} characters"
        ));
    }
    Ok(())
}

pub(super) async fn ensure_target(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    input: &ProvenanceInput,
) -> Result<(), String> {
    semantic_nodes::require(
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
    validate_target_kind(&input.target_kind)?;
    if input.source_revision.trim().is_empty() {
        return Err("Provenance source revision is required".into());
    }
    locator::validate(&input.locator, Some(&input.source_revision))?;
    validate_quote(&input.quote)
}
