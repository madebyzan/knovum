use serde_json::{json, Value};
use sqlx::{Sqlite, Transaction};

use super::{decode::*, package_assets::PackageAssets};
use crate::spaces::db;
mod runtime;
mod source_separation;
#[cfg(test)]
mod tests;
mod translation;

pub(super) struct SpeechSnapshot {
    pub voice_profiles: Vec<Value>,
    pub voice_revisions: Vec<Value>,
    pub voice_candidates: Vec<Value>,
    pub speech_projects: Vec<Value>,
    pub project_revisions: Vec<Value>,
    pub segments: Vec<Value>,
    pub renders: Vec<Value>,
    pub source_separations: Vec<Value>,
}

pub(super) async fn load(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    assets: &mut PackageAssets,
) -> Result<SpeechSnapshot, String> {
    reject_active_work(tx, space_id).await?;
    runtime::validate_completed_runs(tx, space_id).await?;
    translation::validate_completed_runs(tx, space_id).await?;
    let profiles =
        sqlx::query("SELECT * FROM spaces_voice_profiles WHERE space_id=? ORDER BY created_at,id")
            .bind(space_id)
            .fetch_all(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    let voice_revisions = sqlx::query("SELECT r.* FROM spaces_voice_revisions r JOIN spaces_voice_profiles p ON p.id=r.profile_id WHERE p.space_id=? ORDER BY p.created_at,r.revision")
        .bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    let voice_candidates = sqlx::query(
        "SELECT * FROM spaces_voice_candidates WHERE space_id=? ORDER BY created_at,id",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let projects =
        sqlx::query("SELECT p.* FROM spaces_speech_projects p JOIN spaces_nodes n ON n.id=p.source_node_id AND n.deleted_at IS NULL WHERE p.space_id=? ORDER BY p.created_at,p.id")
            .bind(space_id)
            .fetch_all(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    let project_revisions = sqlx::query("SELECT r.* FROM spaces_speech_project_revisions r JOIN spaces_speech_projects p ON p.id=r.project_id WHERE p.space_id=? AND EXISTS(SELECT 1 FROM spaces_nodes n WHERE n.id=p.source_node_id AND n.deleted_at IS NULL) ORDER BY p.created_at,r.revision")
        .bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    let segments = sqlx::query("SELECT s.* FROM spaces_speech_segments s JOIN spaces_speech_project_revisions r ON r.id=s.project_revision_id JOIN spaces_speech_projects p ON p.id=r.project_id WHERE p.space_id=? AND EXISTS(SELECT 1 FROM spaces_nodes n WHERE n.id=p.source_node_id AND n.deleted_at IS NULL) ORDER BY p.created_at,r.revision,s.ordinal")
        .bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;
    let renders = sqlx::query("SELECT o.* FROM spaces_speech_renders o JOIN spaces_speech_segments s ON s.id=o.segment_id JOIN spaces_speech_project_revisions r ON r.id=s.project_revision_id JOIN spaces_speech_projects p ON p.id=r.project_id WHERE p.space_id=? AND EXISTS(SELECT 1 FROM spaces_nodes n WHERE n.id=p.source_node_id AND n.deleted_at IS NULL) ORDER BY o.created_at,o.id")
        .bind(space_id).fetch_all(&mut **tx).await.map_err(db::sql_error)?;

    Ok(SpeechSnapshot {
        voice_profiles: profiles.iter().map(profile).collect::<Result<_, _>>()?,
        voice_revisions: voice_revisions
            .iter()
            .map(|row| voice_revision(row, assets))
            .collect::<Result<_, _>>()?,
        voice_candidates: voice_candidates
            .iter()
            .map(|row| voice_candidate(row, assets))
            .collect::<Result<_, _>>()?,
        speech_projects: projects.iter().map(project).collect::<Result<_, _>>()?,
        project_revisions: project_revisions
            .iter()
            .map(|row| project_revision(row, assets))
            .collect::<Result<_, _>>()?,
        segments: segments.iter().map(segment).collect::<Result<_, _>>()?,
        renders: renders
            .iter()
            .map(|row| render(row, assets))
            .collect::<Result<_, _>>()?,
        source_separations: source_separation::load(tx, space_id, assets).await?,
    })
}

fn voice_candidate(
    row: &sqlx::sqlite::SqliteRow,
    assets: &mut PackageAssets,
) -> Result<Value, String> {
    Ok(json!({
        "id":decode_text(row,"id")?, "runId":decode_text(row,"run_id")?,
        "spaceId":decode_text(row,"space_id")?, "sourceKind":decode_text(row,"source_kind")?,
        "language":decode_text(row,"language")?, "sampleText":decode_text(row,"sample_text")?,
        "designDescription":decode_optional_text(row,"design_description")?,
        "referenceText":decode_optional_text(row,"reference_text")?,
        "inputSourceRevision":decode_optional_text(row,"input_source_revision")?,
        "inputByteLength":decode_optional_integer(row,"input_byte_length")?,
        "referenceAssetPath":assets.reference(&decode_text(row,"reference_asset_path")?),
        "referenceSha256":decode_text(row,"reference_sha256")?,
        "referenceByteLength":decode_integer(row,"reference_byte_length")?,
        "previewAssetPath":assets.reference(&decode_text(row,"preview_asset_path")?),
        "previewSha256":decode_text(row,"preview_sha256")?,
        "previewSampleRate":decode_integer(row,"preview_sample_rate")?,
        "previewSampleCount":decode_integer(row,"preview_sample_count")?,
        "modelSpecId":decode_text(row,"model_spec_id")?, "seed":decode_integer(row,"seed")?,
        "synthesisConfig":decode_json(row,"synthesis_config")?,
        "authorization":decode_json(row,"authorization")?, "status":decode_text(row,"status")?,
        "auditionedAt":decode_optional_integer(row,"auditioned_at")?,
        "selectedVoiceProfileId":decode_optional_text(row,"selected_voice_profile_id")?,
        "selectedVoiceRevisionId":decode_optional_text(row,"selected_voice_revision_id")?,
        "selectedAt":decode_optional_integer(row,"selected_at")?,
        "rejectedAt":decode_optional_integer(row,"rejected_at")?, "createdAt":decode_integer(row,"created_at")?,
    }))
}

async fn reject_active_work(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<(), String> {
    let active_projects: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_runs WHERE home_space_id=? AND completed_at IS NULL \
         AND json_extract(input_snapshot,'$.workflow') IN \
           ('speech_project_generation_v1','speech_source_separation_v1','speech_voice_candidate_v1')",
    )
    .bind(space_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let active: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_speech_segments s JOIN spaces_speech_project_revisions r ON r.id=s.project_revision_id JOIN spaces_speech_projects p ON p.id=r.project_id WHERE p.space_id=? AND s.state='rendering'")
        .bind(space_id).fetch_one(&mut **tx).await.map_err(db::sql_error)?;
    let staged: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_speech_renders o JOIN spaces_speech_segments s ON s.id=o.segment_id JOIN spaces_speech_project_revisions r ON r.id=s.project_revision_id JOIN spaces_speech_projects p ON p.id=r.project_id WHERE p.space_id=? AND o.state='staged'")
        .bind(space_id).fetch_one(&mut **tx).await.map_err(db::sql_error)?;
    let pending_assets: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_speech_asset_publications WHERE space_id=?",
    )
    .bind(space_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if active_projects != 0 || active != 0 || staged != 0 || pending_assets != 0 {
        return Err("Active speech rendering must finish or be cancelled before export".into());
    }
    Ok(())
}

fn profile(row: &sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    Ok(json!({
        "id":decode_text(row,"id")?, "spaceId":decode_text(row,"space_id")?, "name":decode_text(row,"name")?,
        "sourceKind":decode_text(row,"source_kind")?, "activeRevisionId":decode_optional_text(row,"active_revision_id")?,
        "archivedAt":decode_optional_integer(row,"archived_at")?,
        "createdAt":decode_integer(row,"created_at")?, "updatedAt":decode_integer(row,"updated_at")?,
    }))
}

fn voice_revision(
    row: &sqlx::sqlite::SqliteRow,
    assets: &mut PackageAssets,
) -> Result<Value, String> {
    let reference =
        decode_optional_text(row, "reference_asset_id")?.map(|path| assets.reference(&path));
    Ok(json!({
        "id":decode_text(row,"id")?, "profileId":decode_text(row,"profile_id")?, "revision":decode_integer(row,"revision")?,
        "engineId":decode_text(row,"engine_id")?, "modelSpecId":decode_text(row,"model_spec_id")?, "language":decode_text(row,"language")?,
        "presetVoiceId":decode_optional_text(row,"preset_voice_id")?, "referenceAssetId":reference,
        "referenceSourceRevision":decode_optional_text(row,"reference_source_revision")?, "referenceText":decode_optional_text(row,"reference_text")?,
        "designDescription":decode_optional_text(row,"design_description")?, "synthesisConfig":decode_json(row,"synthesis_config")?,
        "authorization":decode_json(row,"authorization")?, "createdAt":decode_integer(row,"created_at")?,
    }))
}

fn project(row: &sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    Ok(json!({
        "sourceNodeId":decode_text(row,"source_node_id")?,
        "id":decode_text(row,"id")?, "spaceId":decode_text(row,"space_id")?, "projectType":decode_text(row,"project_type")?,
        "title":decode_text(row,"title")?, "activeRevisionId":decode_optional_text(row,"active_revision_id")?,
        "createdAt":decode_integer(row,"created_at")?, "updatedAt":decode_integer(row,"updated_at")?,
    }))
}

fn project_revision(
    row: &sqlx::sqlite::SqliteRow,
    assets: &mut PackageAssets,
) -> Result<Value, String> {
    let output =
        decode_optional_text(row, "output_asset_path")?.map(|path| assets.reference(&path));
    let mut config = decode_json(row, "output_config")?;
    if config.get("video").is_some() {
        crate::spaces::speech::video_config::parse(&config)?;
        for key in ["video", "background"] {
            if let Some(path) = config
                .get(key)
                .and_then(|source| source.get("assetPath"))
                .and_then(Value::as_str)
            {
                config[key]["assetPath"] = json!(assets.reference(path));
            }
        }
    }
    if let Some(cover) = config
        .get("metadata")
        .and_then(|metadata| metadata.get("cover"))
        .filter(|value| !value.is_null())
    {
        let path = cover
            .get("assetPath")
            .and_then(Value::as_str)
            .ok_or("Audiobook cover asset is unavailable")?;
        config["metadata"]["cover"]["assetPath"] = json!(assets.reference(path));
    }
    Ok(json!({
        "id":decode_text(row,"id")?, "projectId":decode_text(row,"project_id")?, "revision":decode_integer(row,"revision")?,
        "sourceRevision":decode_text(row,"source_revision")?, "pronunciationLexicon":decode_json(row,"pronunciation_lexicon")?,
        "outputConfig":config, "outputAssetPath":output,
        "outputSha256":decode_optional_text(row,"output_sha256")?, "outputSampleRate":decode_optional_integer(row,"output_sample_rate")?,
        "outputSampleCount":decode_optional_integer(row,"output_sample_count")?,
        "outputPublishedAt":decode_optional_integer(row,"output_published_at")?, "createdAt":decode_integer(row,"created_at")?,
    }))
}

fn segment(row: &sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    Ok(json!({
        "id":decode_text(row,"id")?, "projectRevisionId":decode_text(row,"project_revision_id")?, "ordinal":decode_integer(row,"ordinal")?,
        "chapterKey":decode_optional_text(row,"chapter_key")?, "speakerKey":decode_optional_text(row,"speaker_key")?,
        "text":decode_text(row,"text")?, "normalizedText":decode_text(row,"normalized_text")?, "voiceRevisionId":decode_text(row,"voice_revision_id")?,
        "language":decode_text(row,"language")?, "targetStartMs":decode_optional_integer(row,"target_start_ms")?,
        "targetEndMs":decode_optional_integer(row,"target_end_ms")?, "sourceSegmentId":decode_optional_text(row,"source_segment_id")?,
        "sourceSegmentRevision":decode_optional_integer(row,"source_segment_revision")?, "sourceText":decode_optional_text(row,"source_text")?,
        "state":decode_text(row,"state")?, "stateReason":decode_optional_text(row,"state_reason")?,
        "createdAt":decode_integer(row,"created_at")?, "updatedAt":decode_integer(row,"updated_at")?,
    }))
}

fn render(row: &sqlx::sqlite::SqliteRow, assets: &mut PackageAssets) -> Result<Value, String> {
    let asset_path = decode_optional_text(row, "asset_path")?.map(|path| assets.reference(&path));
    Ok(json!({
        "id":decode_text(row,"id")?, "segmentId":decode_text(row,"segment_id")?, "attemptEpoch":decode_integer(row,"attempt_epoch")?,
        "inputDigest":decode_text(row,"input_digest")?, "modelSpecId":decode_text(row,"model_spec_id")?, "engineVersion":decode_text(row,"engine_version")?,
        "seed":decode_optional_integer(row,"seed")?, "synthesisConfig":decode_json(row,"synthesis_config")?, "state":decode_text(row,"state")?,
        "assetPath":asset_path, "outputSha256":decode_optional_text(row,"output_sha256")?, "sampleRate":decode_optional_integer(row,"sample_rate")?,
        "sampleCount":decode_optional_integer(row,"sample_count")?, "createdAt":decode_integer(row,"created_at")?,
        "publishedAt":decode_optional_integer(row,"published_at")?,
    }))
}
