//! Settling a speech Run's output onto the node reserved for it.
//!
//! The node, its place on the canvas and the lineage that explains it were established when the
//! Run was created. What is left here is content: the published file, the cover it shows, and what
//! it keeps of its source.

use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use super::super::contracts::SpeechAudioOutput;
use crate::spaces::{
    assets::published_asset::PublishedAsset,
    db,
    derivation::{self, inheritance::DerivedOutput},
    knowledge::model::revision,
    previews::GeneratedPreview,
};

pub(in crate::spaces::speech) struct MediaNodeOutput {
    pub node_id: String,
    pub output: SpeechAudioOutput,
    pub derived: DerivedOutput,
    /// The cover a published video shows, made from its own file.
    pub cover: Option<GeneratedPreview>,
}

pub(in crate::spaces::speech) async fn settle(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    source_node_id: &str,
    outputs: &[MediaNodeOutput],
    enrichment_target: Option<&crate::ai::tasks::TaskTargetSelection>,
) -> Result<(), String> {
    let source = node_data(tx, space_id, source_node_id)
        .await?
        .ok_or("Speech source node is no longer available")?;
    let now = crate::foundation::clock::unix_time_millis()?;
    for output in outputs {
        let mut data = node_data(tx, space_id, &output.node_id)
            .await?
            .ok_or("Speech output node is no longer reserved")?;
        if data.get("processingState").is_none() {
            require_same_output(&data, &output.output)?;
            continue;
        }
        apply_published(&mut data, output, &source, now)?;
        let kind = data["kind"]
            .as_str()
            .ok_or("Speech output node kind is invalid")?
            .to_owned();
        crate::spaces::knowledge::validation::validate_node(tx, space_id, &kind, &data).await?;
        derivation::persist_output_node(tx, &output.node_id, &data, now, enrichment_target)
            .await
            .map_err(|error| error.message)?;
    }
    sqlx::query("UPDATE spaces SET updated_at=? WHERE id=?")
        .bind(now)
        .bind(space_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    Ok(())
}

fn apply_published(
    data: &mut Value,
    output: &MediaNodeOutput,
    source: &Value,
    now: i64,
) -> Result<(), String> {
    let fields = data
        .as_object_mut()
        .ok_or("Speech output node data is invalid")?;
    PublishedAsset {
        path: &output.output.path,
        revision: revision_of(&output.output),
        // Encoded audio reports its length in samples rather than bytes.
        size: None,
        cover: output.cover.as_ref(),
    }
    .settle_onto(fields, now)?;
    fields.insert(
        "durationSeconds".into(),
        (output.output.sample_count / u64::from(output.output.sample_rate)).into(),
    );
    output.derived.inherit(data, source)?;
    revision::apply(data)?;
    Ok(())
}

/// A publication that settled in an earlier attempt stands; a different one would mean this Run's
/// node already belongs to another output.
fn require_same_output(data: &Value, output: &SpeechAudioOutput) -> Result<(), String> {
    if data.get("assetPath").and_then(Value::as_str) != Some(output.path.as_str())
        || data.get("sourceRevision").and_then(Value::as_str) != Some(revision_of(output).as_str())
    {
        return Err("Speech Run already published a different node output".into());
    }
    Ok(())
}

fn revision_of(output: &SpeechAudioOutput) -> String {
    format!("sha256:{}", output.sha256)
}

async fn node_data(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    node_id: &str,
) -> Result<Option<Value>, String> {
    let encoded: Option<String> = sqlx::query_scalar(
        "SELECT data FROM spaces_nodes WHERE id=? AND space_id=? AND deleted_at IS NULL",
    )
    .bind(node_id)
    .bind(space_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    encoded
        .map(|encoded| serde_json::from_str(&encoded).map_err(db::json_error))
        .transpose()
}

#[cfg(test)]
mod tests;
