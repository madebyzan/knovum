use assembly_cleanup::{finish_assembly, part_path};
mod assembly;
mod assembly_cleanup;
mod inputs;
use output_spec::digest_render_order;
mod output_spec;
pub use commit::commit_project_output;
use commit::{reusable_committed_output, CommittedOutputSpec};
mod commit;
mod node;
use crate::spaces::speech::asset_publication::PublicationOwnerKind;

mod audiobook;
mod encoding;
mod pauses;
#[cfg(test)]
#[path = "tests/production.rs"]
mod production_tests;

use std::fs;

use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use self::encoding::{wav_sample_count, LoudnessMeasurement};
use super::{
    contracts::{SpeechAudioOutput, SpeechSegmentRenderOutput},
    publication::hash_file,
};
use crate::{local_resources::tools, spaces::runs::AttemptClaim};
pub(super) use audiobook::{
    cleanup_audiobook_checkpoints, measure_audiobook, publish_audiobook,
    publish_chapter_checkpoint, AudiobookInput, ChapterCheckpoint, ChapterPublicationRequest,
    LoudnessCheckpoint,
};

pub(in crate::spaces::speech) struct ProjectOutputRequest<'a> {
    pub project_id: &'a str,
    pub revision_id: &'a str,
    pub project_type: &'a str,
    pub output_config: &'a serde_json::Value,
    pub chapter_keys: &'a [Option<String>],
    pub renders: &'a [SpeechSegmentRenderOutput],
}

pub(in crate::spaces::speech) async fn publish_project<R: Runtime>(
    app: &AppHandle<R>,
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
    request: &ProjectOutputRequest<'_>,
    cancellation: &CancellationToken,
) -> Result<SpeechAudioOutput, String> {
    publish_project_with_measurement(app, pool, claim, request, None, cancellation).await
}

async fn publish_project_with_measurement<R: Runtime>(
    app: &AppHandle<R>,
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
    request: &ProjectOutputRequest<'_>,
    measurement: Option<LoudnessMeasurement>,
    cancellation: &CancellationToken,
) -> Result<SpeechAudioOutput, String> {
    let project_id = request.project_id;
    let revision_id = request.revision_id;
    let renders = request.renders;
    let inputs::ProjectInputs {
        profile,
        digest,
        directory,
    } = inputs::prepare(app, claim, request)?;
    let format = &profile.format;
    let sample_rate = profile.sample_rate;
    let wav_precision = profile.wav_precision;
    if let Some(output) = reusable_committed_output(
        app,
        pool,
        CommittedOutputSpec {
            project_id,
            revision_id,
            digest: &digest,
            format,
            sample_rate,
            wav_precision,
        },
    )
    .await?
    {
        if cancellation.is_cancelled() {
            return Err("cancelled".into());
        }
        return Ok(output);
    }
    let output_path = directory.join(format!("full-{digest}-{}.{format}", claim.attempt_id));
    let sources = inputs::authorize_sources(app, renders, cancellation)?;
    let ffmpeg = tools::resolve_tool(app, "KNOVUM_FFMPEG_PATH", tools::ffmpeg_name())?
        .ok_or("The bundled FFmpeg tool is unavailable")?;
    let work_directory = directory.join(format!("assembly-{}", claim.attempt_id));
    if work_directory.exists() {
        fs::remove_dir_all(&work_directory).map_err(|error| error.to_string())?;
    }
    fs::create_dir_all(&work_directory).map_err(|error| error.to_string())?;
    let part = part_path(&output_path);
    let mut publication_id = None;
    let assembled: Result<SpeechAudioOutput, String> = async {
        if part.exists() {
            fs::remove_file(&part).map_err(|error| error.to_string())?;
        }
        let input = assembly::AssemblyInput {
            ffmpeg,
            sources: sources.clone(),
            renders: renders.to_vec(),
            work_directory: work_directory.clone(),
            output: part.clone(),
            profile: profile.clone(),
            chapter_keys: request.chapter_keys.to_vec(),
            measurement,
        };
        let assembly_cancellation = cancellation.clone();
        let sample_count =
            crate::blocking::run(move || assembly::assemble(&input, &assembly_cancellation))
                .await?;
        inputs::verify_cover(app, &profile)?;
        inputs::verify_sources(&sources, renders, cancellation)?;
        if cancellation.is_cancelled() {
            return Err("cancelled".into());
        }
        if cfg!(windows) && output_path.exists() {
            fs::remove_file(&output_path).map_err(|error| error.to_string())?;
        }
        let sha256 = hash_file(&part)?;
        let sample_count =
            if output_path.extension().and_then(|value| value.to_str()) == Some("wav") {
                wav_sample_count(&part, sample_rate, wav_precision)?
            } else {
                sample_count
            };
        publication_id = Some(
            super::asset_publication::stage_for_attempt(
                pool,
                claim,
                super::asset_publication::PendingAsset {
                    owner_kind: PublicationOwnerKind::ProjectOutput,
                    owner_id: revision_id,
                    asset_path: &output_path,
                    part_path: Some(&part),
                    sha256: &sha256,
                },
            )
            .await?,
        );
        fs::rename(&part, &output_path).map_err(|error| error.to_string())?;
        if hash_file(&output_path)? != sha256 {
            return Err("Published speech project hash changed during atomic publication".into());
        }
        Ok(SpeechAudioOutput {
            path: output_path.to_string_lossy().into_owned(),
            sha256,
            sample_rate,
            sample_count,
        })
    }
    .await;
    let assembled = finish_assembly(assembled, &part, &work_directory);
    match assembled {
        Ok(output) => Ok(output),
        Err(mut error) => {
            if let Some(publication_id) = publication_id {
                if let Err(cleanup_error) =
                    super::asset_publication::abandon(app, pool, &publication_id).await
                {
                    error.push_str("; failed to abandon project publication: ");
                    error.push_str(&cleanup_error);
                }
            } else if let Err(cleanup_error) = fs::remove_file(&output_path) {
                if cleanup_error.kind() != std::io::ErrorKind::NotFound {
                    error.push_str(&format!(
                        "; failed to remove {}: {cleanup_error}",
                        output_path.display()
                    ));
                }
            }
            Err(error)
        }
    }
}
