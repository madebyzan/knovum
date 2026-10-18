use serde_json::{json, Value};
use sqlx::{Sqlite, Transaction};

use super::super::contracts::SpeechAudioOutput;
use crate::spaces::{
    derivation::{
        inheritance::DerivedOutput,
        placement::{self, DerivedCell, DerivedOutputs},
        relations,
    },
    nodes::{self, contracts::InsertNode},
    previews::GeneratedPreview,
    structure::{self, Size, SizingMode},
};

pub(in crate::spaces::speech) struct MediaNodeOutput {
    pub node_id: String,
    pub kind: &'static str,
    pub title: String,
    pub output: SpeechAudioOutput,
    pub derived: DerivedOutput,
    /// The cover a published video shows, made from its own file.
    pub cover: Option<GeneratedPreview>,
}

impl MediaNodeOutput {
    /// Separated media has no intrinsic canvas size, so the kind decides its footprint.
    fn size(&self) -> Size {
        if self.kind == "audio" {
            Size {
                width: 288.0,
                height: 80.0,
            }
        } else {
            placement::PLACEHOLDER
        }
    }

    fn build_insert_node(&self, cell: &DerivedCell, source: &Value) -> Result<InsertNode, String> {
        let file_name = std::path::Path::new(&self.output.path)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("Speech output has no file name")?;
        let mut data = json!({ "kind":self.kind, "title":self.title, "assetPath":self.output.path,
            "fileName":file_name, "sourceRevision":format!("sha256:{}", self.output.sha256),
            "durationSeconds": self.output.sample_count / u64::from(self.output.sample_rate) });
        if let (Some(cover), Some(object)) = (&self.cover, data.as_object_mut()) {
            cover.settle_onto(object);
        }
        self.derived.inherit(&mut data, source)?;
        Ok(InsertNode {
            id: self.node_id.clone(),
            kind: self.kind.into(),
            initial_x: cell.x,
            initial_y: cell.y,
            width: cell.size.width,
            height: cell.size.height,
            sizing_mode: SizingMode::Content,
            appearance: None,
            file_subpath: None,
            source_chunks: Vec::new(),
            data,
        })
    }
}

pub(in crate::spaces::speech) async fn publish(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    source_node_id: &str,
    outputs: &[MediaNodeOutput],
) -> Result<(), String> {
    let (mut snapshot, edit_revision) =
        structure::repository::load_in_transaction(tx, space_id).await?;
    let before = structure::history::capture(tx, space_id, &snapshot).await?;
    let source: String = sqlx::query_scalar(
        "SELECT data FROM spaces_nodes WHERE id=? AND space_id=? AND deleted_at IS NULL",
    )
    .bind(source_node_id)
    .bind(space_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(crate::spaces::db::sql_error)?
    .ok_or("Speech source node is no longer available")?;
    let source: Value = serde_json::from_str(&source).map_err(crate::spaces::db::json_error)?;
    let sources = [source_node_id.to_owned()];
    let cells = placement::plan(
        tx,
        space_id,
        &snapshot.structure,
        &snapshot.layouts,
        DerivedOutputs {
            sources: &sources,
            reserved: &[],
            sizes: outputs.iter().map(MediaNodeOutput::size).collect(),
        },
    )
    .await
    .map_err(|error| error.to_string())?;
    let mut nodes = Vec::with_capacity(outputs.len());
    for (output, cell) in outputs.iter().zip(&cells) {
        let mut node = output.build_insert_node(cell, &source)?;
        crate::spaces::knowledge::validation::normalize_node(
            tx,
            space_id,
            &node.kind,
            &mut node.data,
        )
        .await?;
        nodes.push(node);
    }
    let now = crate::foundation::clock::unix_time_millis()?;
    structure::creation::append_content(&mut snapshot, &nodes, None)?;
    nodes::insert_rows_with_actor(
        tx,
        space_id,
        &nodes,
        crate::spaces::knowledge::events::EventActor::new("system", None),
        now,
        None,
    )
    .await
    .map_err(|error| error.to_string())?;
    let revision =
        structure::repository::save_in_transaction(tx, space_id, edit_revision, &snapshot)
            .await
            .map_err(|error| error.to_string())?;
    structure::portal_integrity::validate_structure_targets(tx, &[space_id]).await?;
    for output in outputs {
        relations::derives_from(
            tx,
            space_id,
            &output.node_id,
            source_node_id,
            "Media derived from its source node",
            now,
        )
        .await?;
    }
    let after = structure::history::capture(tx, space_id, &snapshot).await?;
    structure::history::record(tx, space_id, &before, &after, revision).await?;
    sqlx::query("UPDATE spaces SET updated_at=? WHERE id=?")
        .bind(now)
        .bind(space_id)
        .execute(&mut **tx)
        .await
        .map_err(crate::spaces::db::sql_error)?;
    Ok(())
}

#[cfg(test)]
mod tests;
