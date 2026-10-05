mod chapter;
mod chapter_cache;
mod chapter_commit;
mod chapter_render;
mod cleanup;
mod loudness;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use super::super::contracts::{SpeechAudioOutput, SpeechSegmentRenderOutput};
use super::{publish_project_with_measurement, ProjectOutputRequest};
use crate::spaces::runs::AttemptClaim;

pub(in crate::spaces::speech) use chapter::publish_chapter_checkpoint;
pub(in crate::spaces::speech) use cleanup::cleanup_audiobook_checkpoints;
pub(in crate::spaces::speech) use loudness::measure_audiobook;

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::spaces::speech) struct ChapterCheckpoint {
    pub chapter_key: String,
    pub path: String,
    pub sha256: String,
    pub sample_rate: u32,
    pub sample_count: u64,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(in crate::spaces::speech) struct LoudnessCheckpoint {
    pub(super) input_digest: String,
    pub(super) measurement: super::encoding::LoudnessMeasurement,
}

pub(in crate::spaces::speech) struct ChapterPublicationRequest<'a> {
    pub project_id: &'a str,
    pub revision_id: &'a str,
    pub chapter_key: &'a str,
    pub sample_rate: u32,
    pub paragraph_pause_ms: u32,
    pub renders: &'a [SpeechSegmentRenderOutput],
}

pub(in crate::spaces::speech) struct AudiobookInput<'a> {
    pub project_id: &'a str,
    pub revision_id: &'a str,
    pub output_config: &'a serde_json::Value,
    pub chapters: &'a [ChapterCheckpoint],
}

pub(in crate::spaces::speech) async fn publish_audiobook<R: Runtime>(
    app: &AppHandle<R>,
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
    input: &AudiobookInput<'_>,
    checkpoint: LoudnessCheckpoint,
    cancellation: &CancellationToken,
) -> Result<SpeechAudioOutput, String> {
    if checkpoint.input_digest != chapter_digest(input.chapters, input.output_config) {
        return Err("Audiobook loudness checkpoint does not match its chapter inputs".into());
    }
    let renders = input
        .chapters
        .iter()
        .map(|chapter| SpeechSegmentRenderOutput {
            segment_id: chapter.chapter_key.clone(),
            path: chapter.path.clone(),
            sha256: chapter.sha256.clone(),
            sample_rate: chapter.sample_rate,
            sample_count: chapter.sample_count,
            cached: true,
        })
        .collect::<Vec<_>>();
    let chapter_keys = input
        .chapters
        .iter()
        .map(|chapter| Some(chapter.chapter_key.clone()))
        .collect::<Vec<_>>();
    publish_project_with_measurement(
        app,
        pool,
        claim,
        &ProjectOutputRequest {
            project_id: input.project_id,
            revision_id: input.revision_id,
            project_type: "audiobook",
            output_config: input.output_config,
            chapter_keys: &chapter_keys,
            renders: &renders,
        },
        Some(checkpoint.measurement),
        cancellation,
    )
    .await
}

pub(super) fn chapter_digest(
    chapters: &[ChapterCheckpoint],
    output_config: &serde_json::Value,
) -> String {
    let mut digest = Sha256::new();
    for chapter in chapters {
        digest.update(chapter.chapter_key.as_bytes());
        digest.update([0]);
        digest.update(chapter.sha256.as_bytes());
        digest.update(chapter.sample_count.to_le_bytes());
    }
    digest.update(output_config.to_string().as_bytes());
    format!("{:x}", digest.finalize())
}
