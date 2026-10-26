use serde::Deserialize;
use sqlx::{Sqlite, Transaction};

mod images;
pub use images::{render_images, write_png_files, RenderedImage};

use super::inputs::{DerivationInput, RevisionBasis};
use super::source_validation;

/// One resource a creation Run was accepted with, pinned to the revision the
/// user saw. The basis travels with the input because it is decided once, when
/// the Run resolves its sources: indexed knowledge for text, the asset itself
/// for pixels that no model has read yet.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceInput {
    pub node_id: String,
    pub role: String,
    pub source_revision: String,
    pub revision_basis: RevisionBasis,
}

impl SourceInput {
    pub fn is_primary(&self) -> bool {
        self.role == "primary"
    }
}

/// The first resource leads, the rest support it.
pub fn require_single_primary(inputs: &[SourceInput]) -> Result<(), String> {
    if inputs.is_empty() {
        return Err(UNLED.into());
    }
    require_led(inputs)
}

/// A generation reads whatever resources it was given, which may be none; any it has are led by
/// the first.
pub fn require_led(inputs: &[SourceInput]) -> Result<(), String> {
    match inputs.first() {
        None => Ok(()),
        Some(first)
            if first.is_primary()
                && inputs.iter().filter(|input| input.is_primary()).count() == 1 =>
        {
            Ok(())
        }
        Some(_) => Err(UNLED.into()),
    }
}

const UNLED: &str = "Creation sources must start with exactly one primary resource";

/// Reject the Run when a source changed after it was accepted, so a queued Run
/// can never send content the user has already replaced.
pub async fn pin(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    inputs: &[SourceInput],
) -> Result<(), String> {
    for input in inputs {
        source_validation::validate(
            tx,
            space_id,
            &input.node_id,
            &input.source_revision,
            input.revision_basis,
        )
        .await?;
    }
    Ok(())
}

/// Pin every source and confirm the reserved output belongs to the same Space.
pub async fn pin_for_output(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    output_node_id: &str,
    inputs: &[SourceInput],
) -> Result<(), String> {
    for input in inputs {
        crate::spaces::knowledge::validation::validate_same_space_node(
            tx,
            space_id,
            &input.node_id,
            output_node_id,
        )
        .await?;
    }
    pin(tx, space_id, inputs).await
}

pub fn records(inputs: &[SourceInput]) -> Vec<DerivationInput<'_>> {
    inputs
        .iter()
        .map(|input| DerivationInput {
            node_id: &input.node_id,
            role: &input.role,
            source_revision: &input.source_revision,
            revision_basis: input.revision_basis,
        })
        .collect()
}

pub fn node_ids(inputs: &[SourceInput]) -> Vec<String> {
    inputs.iter().map(|input| input.node_id.clone()).collect()
}

#[cfg(test)]
mod tests;
