use serde_json::json;
use sqlx::{Sqlite, Transaction};

use super::super::contracts::SpeechAudioOutput;
use crate::spaces::{
    derivation::relations,
    nodes::{self, contracts::InsertNode},
    structure::{self, SizingMode},
};

const OUTPUT_GAP: f64 = 48.0;

pub(in crate::spaces::speech) struct MediaNodeOutput {
    pub node_id: String,
    pub kind: &'static str,
    pub title: String,
    pub output: SpeechAudioOutput,
}

impl MediaNodeOutput {
    fn size(&self) -> (f64, f64) {
        if self.kind == "audio" {
            (288.0, 80.0)
        } else {
            (320.0, 220.0)
        }
    }

    fn build_insert_node(&self, x: f64, y: f64) -> Result<InsertNode, String> {
        let file_name = std::path::Path::new(&self.output.path)
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("Speech output has no file name")?;
        let (width, height) = self.size();
        Ok(InsertNode {
            id: self.node_id.clone(),
            kind: self.kind.into(),
            initial_x: x,
            initial_y: y,
            width,
            height,
            sizing_mode: SizingMode::Content,
            appearance: None,
            file_subpath: None,
            source_chunks: Vec::new(),
            data: json!({ "kind":self.kind, "title":self.title, "assetPath":self.output.path,
                "fileName":file_name, "sourceRevision":format!("sha256:{}", self.output.sha256),
                "durationSeconds": self.output.sample_count / u64::from(self.output.sample_rate) }),
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
    let width = outputs
        .iter()
        .map(|output| output.size().0)
        .fold(0.0, f64::max);
    let height = outputs
        .iter()
        .map(|output| output.size().1 + OUTPUT_GAP)
        .sum::<f64>()
        - OUTPUT_GAP;
    let (x, mut y) = structure::place_new_content(
        &snapshot.structure,
        &snapshot.layouts,
        &[source_node_id.into()],
        width,
        height,
    )?;
    let mut nodes = Vec::with_capacity(outputs.len());
    for output in outputs {
        let mut node = output.build_insert_node(x, y)?;
        crate::spaces::knowledge::validation::normalize_node(
            tx,
            space_id,
            &node.kind,
            &mut node.data,
        )
        .await?;
        nodes.push(node);
        y += output.size().1 + OUTPUT_GAP;
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
        relations::insert(
            tx,
            &relations::derives_from(
                space_id,
                &output.node_id,
                source_node_id,
                "Media derived from its source node",
                now,
            ),
        )
        .await?;
    }
    let after = structure::history::capture(tx, space_id, &snapshot).await?;
    structure::history::record(tx, space_id, &before, &after, None, None, revision).await?;
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
