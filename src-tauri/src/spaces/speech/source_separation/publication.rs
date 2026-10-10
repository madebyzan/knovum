use sqlx::{Sqlite, Transaction};
use std::path::Path;

use super::SeparationConfiguration;
use crate::{
    ai::local_runtime::separation::SeparatedAudio,
    spaces::{
        runs::AttemptClaim,
        speech::{
            contracts::SpeechAudioOutput,
            publication::nodes::{self, MediaNodeOutput},
        },
    },
};

pub(super) async fn publish(
    tx: &mut Transaction<'_, Sqlite>,
    claim: &AttemptClaim,
    config: &SeparationConfiguration,
    background: &Path,
    vocals: &Path,
    metadata: &SeparatedAudio,
) -> Result<(), String> {
    let space_id = claim.scope.require_space_id()?;
    let title =
        super::media::require_owned_source(tx, space_id, &config.source_node_id, &config.source)
            .await?;
    let outputs = [
        ("vocals", &config.vocals_label, vocals, &metadata.vocals),
        (
            "background",
            &config.background_label,
            background,
            &metadata.background,
        ),
    ]
    .map(|(stem, label, path, audio)| MediaNodeOutput {
        node_id: format!("{}-{stem}", claim.run_id),
        kind: "audio",
        title: format!("{title} · {label}"),
        output: SpeechAudioOutput {
            path: path.to_string_lossy().into_owned(),
            sha256: audio.sha256.clone(),
            sample_rate: audio.sample_rate,
            sample_count: audio.sample_count,
        },
    });
    nodes::publish(tx, space_id, &config.source_node_id, &outputs).await
}
