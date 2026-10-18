//! The nodes a speech Run publishes into.
//!
//! A Run that will put content on the canvas reserves its node as it is created, so the work is
//! visible — and connected to what it was made from — from the moment it is asked for rather than
//! only once it succeeds. Reservation, placement and lineage belong to the derivation contract,
//! which is why this only names what speech produces.

use std::path::{Path, PathBuf};

use sqlx::{Sqlite, Transaction};
use tauri::AppHandle;

use crate::{
    ai::tasks::TaskTargetSelection,
    spaces::{
        derivation::{
            output_paths,
            pending_output::{self, PendingOutput},
        },
        structure::Size,
    },
};

/// Audio has no picture to size itself by, so the canvas gives it a strip.
const AUDIO_FOOTPRINT: Size = Size {
    width: 288.0,
    height: 80.0,
};

/// What reserving an output needs of the application: where managed assets live, and who would
/// read the published content. Resolved once, where the Run is asked for.
pub(super) struct OutputContext {
    asset_directory: PathBuf,
    enrichment_target: Option<TaskTargetSelection>,
}

impl OutputContext {
    pub(super) fn resolve(app: &AppHandle) -> Result<Self, String> {
        Ok(Self {
            asset_directory: crate::spaces::assets::asset_directory(app)?,
            enrichment_target: crate::spaces::assets::enrichment::jobs::intent::selection(app)?,
        })
    }

    #[cfg(test)]
    pub(super) fn for_tests(asset_directory: PathBuf) -> Self {
        Self {
            asset_directory,
            enrichment_target: None,
        }
    }

    fn directory(&self) -> &Path {
        &self.asset_directory
    }
}

/// What a speech Run will publish. The file itself is named by the publication that writes it;
/// this is the node that waits for it.
pub(super) struct ReservedOutput {
    pub node_id: String,
    pub kind: &'static str,
    pub title: String,
    pub extension: String,
}

pub(super) async fn reserve(
    context: &OutputContext,
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    run_id: &str,
    source_node_id: &str,
    outputs: Vec<ReservedOutput>,
    lineage: &'static str,
) -> Result<(), String> {
    let pending = outputs
        .into_iter()
        .map(|output| PendingOutput {
            data: serde_json::json!({
                "kind": output.kind,
                "title": output.title,
                "fileName": format!("{}.{}", output.title, output.extension),
                "assetPath": output_paths::reserve_in(
                    context.directory(), "speech", &output.extension,
                ),
                "processingState": "processing",
            }),
            footprint: if output.kind == "audio" {
                AUDIO_FOOTPRINT
            } else {
                pending_output::PLACEHOLDER
            },
            node_id: output.node_id,
            kind: output.kind,
        })
        .collect();
    pending_output::reserve_in_transaction(
        tx,
        space_id,
        pending,
        &[source_node_id.to_owned()],
        lineage,
        run_id,
        context.enrichment_target.as_ref(),
    )
    .await
    .map_err(|error| error.message)
}
