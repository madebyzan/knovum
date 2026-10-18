use sqlx::{Sqlite, Transaction};
use std::path::Path;

use super::SeparationConfiguration;
use crate::{
    ai::local_runtime::separation::SeparatedAudio,
    spaces::{
        derivation::inheritance::DerivedOutput,
        runs::AttemptClaim,
        speech::{
            contracts::SpeechAudioOutput,
            output_reservation::{self, OutputContext, ReservedOutput},
            publication::nodes::{self, MediaNodeOutput},
        },
    },
};

/// The two tracks separation pulls apart, in the order they are reserved and settled.
const STEMS: [(&str, DerivedOutput); 2] = [
    ("vocals", DerivedOutput::SeparatedVocals),
    ("background", DerivedOutput::SeparatedBackground),
];

fn node_id(run_id: &str, stem: &str) -> String {
    format!("{run_id}-{stem}")
}

pub(super) async fn reserve_outputs(
    context: &OutputContext,
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    run_id: &str,
    source_node_id: &str,
    labels: [&str; 2],
) -> Result<(), String> {
    let title: String = sqlx::query_scalar(
        "SELECT json_extract(data,'$.title') FROM spaces_nodes \
         WHERE id=? AND space_id=? AND deleted_at IS NULL",
    )
    .bind(source_node_id)
    .bind(space_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(crate::spaces::db::sql_error)?
    .ok_or("Source separation source node is no longer available")?;
    let outputs = STEMS
        .iter()
        .zip(labels)
        .map(|((stem, _), label)| ReservedOutput {
            node_id: node_id(run_id, stem),
            kind: "audio",
            title: format!("{title} · {label}"),
            extension: "wav".to_owned(),
        })
        .collect();
    output_reservation::reserve(
        context,
        tx,
        space_id,
        run_id,
        source_node_id,
        outputs,
        "Track separated from its source media",
    )
    .await
}

pub(super) async fn publish(
    tx: &mut Transaction<'_, Sqlite>,
    claim: &AttemptClaim,
    config: &SeparationConfiguration,
    background: &Path,
    vocals: &Path,
    metadata: &SeparatedAudio,
    enrichment_target: Option<&crate::ai::tasks::TaskTargetSelection>,
) -> Result<(), String> {
    let space_id = claim.scope.require_space_id()?;
    super::media::require_owned_source(tx, space_id, &config.source_node_id, &config.source)
        .await?;
    let outputs = STEMS
        .iter()
        .zip([
            (vocals, &metadata.vocals),
            (background, &metadata.background),
        ])
        .map(|((stem, derived), (path, audio))| MediaNodeOutput {
            node_id: node_id(&claim.run_id, stem),
            output: SpeechAudioOutput {
                path: path.to_string_lossy().into_owned(),
                sha256: audio.sha256.clone(),
                sample_rate: audio.sample_rate,
                sample_count: audio.sample_count,
            },
            derived: *derived,
            cover: None,
        })
        .collect::<Vec<_>>();
    nodes::settle(
        tx,
        space_id,
        &config.source_node_id,
        &outputs,
        enrichment_target,
    )
    .await
}
