use assembly_cleanup::{finish_assembly, part_path};
mod assembly_cleanup;
use output_spec::{digest_render_order, output_profile};
mod output_spec;
pub use commit::commit_project_output;
use commit::{reusable_committed_output, CommittedOutputSpec};
mod commit;
use crate::spaces::speech::asset_publication::PublicationOwnerKind;

mod audiobook;
mod encoding;
mod pauses;
#[cfg(test)]
#[path = "tests/production.rs"]
mod production_tests;

use std::{fs, path::Path};

use tauri::{AppHandle, Runtime};
use tokio_util::sync::CancellationToken;

use self::encoding::{
    encode, normalize_sources, wav_sample_count, write_chapter_metadata, write_concat_file,
    EncodeRequest, LoudnessMeasurement,
};
use super::{
    contracts::{SpeechAudioOutput, SpeechSegmentRenderOutput},
    publication::hash_file,
};
use crate::{
    local_resources::tools,
    spaces::{assets, db, runs::AttemptClaim},
};
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
    let project_type = request.project_type;
    let output_config = request.output_config;
    let chapter_keys = request.chapter_keys;
    let renders = request.renders;
    uuid::Uuid::parse_str(project_id).map_err(|_| "Speech project identity is invalid")?;
    uuid::Uuid::parse_str(revision_id).map_err(|_| "Speech revision identity is invalid")?;
    uuid::Uuid::parse_str(&claim.attempt_id)
        .map_err(|_| "Speech assembly attempt identity is invalid")?;
    if renders.is_empty() || chapter_keys.len() != renders.len() {
        return Err("Speech project has no complete ordered render set".into());
    }
    let (format, sample_rate, audiobook) = output_profile(project_type, output_config, renders)?;
    let wav_precision = if let Some(config) = &audiobook {
        config.wav_precision
    } else {
        serde_json::from_value::<super::project_config::ShortOutputConfig>(output_config.clone())
            .map_err(db::json_error)?
            .wav_precision
    };
    if let Some(config) = audiobook.as_ref() {
        super::project_config::validate_chapter_order(
            config,
            chapter_keys.iter().map(Option::as_deref),
        )?;
    }
    if let Some(cover) = audiobook
        .as_ref()
        .and_then(|config| config.metadata.cover.as_ref())
    {
        let path = assets::authorize_spaces_asset(app, Path::new(&cover.asset_path))?;
        crate::file_inspection::verify_content_revision(&path, &cover.source_revision)?;
    }
    let digest = digest_render_order(renders, output_config);
    let directory = assets::asset_directory(app)?
        .join("speech")
        .join(project_id)
        .join(revision_id);
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    if let Some(output) = reusable_committed_output(
        app,
        pool,
        CommittedOutputSpec {
            project_id,
            revision_id,
            digest: &digest,
            format: &format,
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
    let sources = renders
        .iter()
        .map(|render| {
            if cancellation.is_cancelled() {
                return Err("cancelled".into());
            }
            let path = assets::authorize_spaces_asset(app, Path::new(&render.path))?;
            if hash_file(&path)? != render.sha256 {
                return Err("Speech segment digest differs from its render metadata".into());
            }
            Ok(path)
        })
        .collect::<Result<Vec<_>, String>>()?;
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
        let normalize_ffmpeg = ffmpeg.clone();
        let input_sources = sources.clone();
        let normalize_renders = renders.to_vec();
        let normalize_directory = work_directory.clone();
        let normalize_cancellation = cancellation.clone();
        let mut normalized = crate::blocking::run(move || {
            normalize_sources(
                &normalize_ffmpeg,
                &input_sources,
                &normalize_renders,
                sample_rate,
                &normalize_directory,
                &normalize_cancellation,
            )
        })
        .await?;
        if let Some(config) = &audiobook {
            let pause_directory = work_directory.clone();
            let pause_cancellation = cancellation.clone();
            let pause_ms = config.chapter_pause_ms;
            normalized = crate::blocking::run(move || {
                pauses::insert(
                    &normalized,
                    sample_rate,
                    pause_ms,
                    &pause_directory,
                    &pause_cancellation,
                )
            })
            .await?;
        }
        let sample_count = normalized.iter().try_fold(0_u64, |total, source| {
            total
                .checked_add(source.sample_count)
                .ok_or_else(|| "Speech project sample count overflow".to_owned())
        })?;
        let concat_path = write_concat_file(&work_directory, &normalized)?;
        let metadata_chapter_keys = if audiobook
            .as_ref()
            .is_some_and(|config| config.chapter_pause_ms > 0)
        {
            chapter_keys
                .iter()
                .enumerate()
                .flat_map(|(index, key)| {
                    let mut keys = Vec::new();
                    if index > 0 {
                        keys.push(chapter_keys[index - 1].clone());
                    }
                    keys.push(key.clone());
                    keys
                })
                .collect::<Vec<_>>()
        } else {
            chapter_keys.to_vec()
        };
        let metadata_path = if let Some(config) = audiobook.as_ref() {
            Some(write_chapter_metadata(
                &work_directory,
                config,
                &metadata_chapter_keys,
                &normalized,
                sample_rate,
            )?)
        } else {
            None
        };
        let encode_ffmpeg = ffmpeg;
        let encode_concat = concat_path;
        let encode_output = part.clone();
        let encode_format = format;
        let encode_audiobook = audiobook.clone();
        let encode_metadata = metadata_path;
        let encode_cancellation = cancellation.clone();
        crate::blocking::run(move || {
            encode(
                EncodeRequest {
                    ffmpeg: &encode_ffmpeg,
                    concat_path: &encode_concat,
                    output: &encode_output,
                    format: &encode_format,
                    sample_rate,
                    wav_precision,
                    audiobook: encode_audiobook.as_ref(),
                    measurement: measurement.as_ref(),
                    metadata: encode_metadata.as_deref(),
                },
                &encode_cancellation,
            )
        })
        .await?;
        if let Some(cover) = audiobook
            .as_ref()
            .and_then(|config| config.metadata.cover.as_ref())
        {
            crate::file_inspection::verify_content_revision(
                Path::new(&cover.asset_path),
                &cover.source_revision,
            )?;
        }
        for (source, render) in sources.iter().zip(renders) {
            if cancellation.is_cancelled() {
                return Err("cancelled".into());
            }
            if hash_file(source)? != render.sha256 {
                return Err("Speech segment changed while the project was being encoded".into());
            }
        }
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
