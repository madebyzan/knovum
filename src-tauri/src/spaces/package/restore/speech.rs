use sqlx::{Sqlite, Transaction};

use super::contracts::RestorePackageRequest;
use crate::spaces::db;

pub(super) async fn insert(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    insert_voices(tx, request).await?;
    insert_voice_candidates(tx, request).await?;
    insert_source_separations(tx, request).await?;
    insert_projects(tx, request).await
}

async fn insert_voice_candidates(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.voice_candidates {
        sqlx::query("INSERT INTO spaces_voice_candidates(id,space_id,run_id,source_kind,language,sample_text,design_description,reference_text,input_source_revision,input_byte_length,reference_asset_path,reference_sha256,reference_byte_length,preview_asset_path,preview_sha256,preview_sample_rate,preview_sample_count,model_spec_id,seed,synthesis_config,authorization,status,auditioned_at,selected_voice_profile_id,selected_voice_revision_id,selected_at,rejected_at,created_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)")
            .bind(&row.id).bind(&request.space.id).bind(&row.run_id).bind(&row.source_kind)
            .bind(&row.language).bind(&row.sample_text).bind(&row.design_description).bind(&row.reference_text)
            .bind(&row.input_source_revision).bind(row.input_byte_length)
            .bind(&row.reference_asset_path).bind(&row.reference_sha256).bind(row.reference_byte_length)
            .bind(&row.preview_asset_path).bind(&row.preview_sha256).bind(row.preview_sample_rate)
            .bind(row.preview_sample_count).bind(&row.model_spec_id).bind(row.seed)
            .bind(json(&row.synthesis_config)?).bind(json(&row.authorization)?).bind(&row.status)
            .bind(row.auditioned_at).bind(&row.selected_voice_profile_id).bind(&row.selected_voice_revision_id)
            .bind(row.selected_at).bind(row.rejected_at).bind(row.created_at).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

async fn insert_source_separations(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.speech_source_separations {
        sqlx::query("INSERT INTO spaces_speech_source_separations(id,space_id,run_id,source_asset_path,source_revision,source_duration_ms,model_spec_id,background_asset_path,background_sha256,vocals_asset_path,vocals_sha256,background_sample_rate,background_sample_count,background_duration_ms,review_status,review_warning,created_at,reviewed_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)")
            .bind(&row.id).bind(&request.space.id).bind(&row.run_id).bind(&row.source_asset_path)
            .bind(&row.source_revision).bind(row.source_duration_ms).bind(&row.model_spec_id)
            .bind(&row.background_asset_path).bind(&row.background_sha256).bind(&row.vocals_asset_path).bind(&row.vocals_sha256).bind(row.background_sample_rate)
            .bind(row.background_sample_count).bind(row.background_duration_ms).bind(&row.review_status)
            .bind(&row.review_warning).bind(row.created_at).bind(row.reviewed_at)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

async fn insert_voices(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.voice_profiles {
        sqlx::query("INSERT INTO spaces_voice_profiles(id,space_id,name,source_kind,active_revision_id,archived_at,created_at,updated_at) VALUES(?,?,?,?,NULL,?,?,?)")
            .bind(&row.id).bind(&request.space.id).bind(&row.name).bind(&row.source_kind)
            .bind(row.archived_at).bind(row.created_at).bind(row.updated_at).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    for row in &request.voice_revisions {
        sqlx::query("INSERT INTO spaces_voice_revisions(id,profile_id,revision,engine_id,model_spec_id,language,preset_voice_id,reference_asset_id,reference_source_revision,reference_text,design_description,synthesis_config,authorization,created_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?)")
            .bind(&row.id).bind(&row.profile_id).bind(row.revision).bind(&row.engine_id).bind(&row.model_spec_id)
            .bind(&row.language).bind(&row.preset_voice_id).bind(&row.reference_asset_id).bind(&row.reference_source_revision)
            .bind(&row.reference_text).bind(&row.design_description).bind(json(&row.synthesis_config)?)
            .bind(json(&row.authorization)?).bind(row.created_at).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    for row in &request.voice_profiles {
        sqlx::query("UPDATE spaces_voice_profiles SET active_revision_id=? WHERE id=?")
            .bind(&row.active_revision_id)
            .bind(&row.id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    }
    Ok(())
}

async fn insert_projects(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.speech_projects {
        sqlx::query("INSERT INTO spaces_speech_projects(id,space_id,source_node_id,project_type,title,active_revision_id,created_at,updated_at) VALUES(?,?,?,?,?,NULL,?,?)")
            .bind(&row.id).bind(&request.space.id).bind(&row.source_node_id).bind(&row.project_type).bind(&row.title)
            .bind(row.created_at).bind(row.updated_at).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    for row in &request.speech_project_revisions {
        sqlx::query("INSERT INTO spaces_speech_project_revisions(id,project_id,revision,source_revision,pronunciation_lexicon,output_config,output_asset_path,output_sha256,output_sample_rate,output_sample_count,output_published_at,created_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,?)")
            .bind(&row.id).bind(&row.project_id).bind(row.revision).bind(&row.source_revision)
            .bind(json(&row.pronunciation_lexicon)?).bind(json(&row.output_config)?).bind(&row.output_asset_path)
            .bind(&row.output_sha256).bind(row.output_sample_rate).bind(row.output_sample_count)
            .bind(row.output_published_at).bind(row.created_at)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    for row in &request.speech_segments {
        sqlx::query("INSERT INTO spaces_speech_segments(id,project_revision_id,ordinal,chapter_key,speaker_key,text,normalized_text,delivery_instruction,voice_revision_id,language,target_start_ms,target_end_ms,source_segment_id,source_segment_revision,source_text,state,state_reason,created_at,updated_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)")
            .bind(&row.id).bind(&row.project_revision_id).bind(row.ordinal).bind(&row.chapter_key).bind(&row.speaker_key)
            .bind(&row.text).bind(&row.normalized_text).bind(&row.delivery_instruction).bind(&row.voice_revision_id).bind(&row.language)
            .bind(row.target_start_ms).bind(row.target_end_ms).bind(&row.source_segment_id)
            .bind(row.source_segment_revision).bind(&row.source_text).bind(&row.state).bind(&row.state_reason)
            .bind(row.created_at).bind(row.updated_at).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    for row in &request.speech_renders {
        sqlx::query("INSERT INTO spaces_speech_renders(id,segment_id,attempt_epoch,input_digest,model_spec_id,engine_version,seed,synthesis_config,state,asset_path,output_sha256,sample_rate,sample_count,created_at,published_at) VALUES(?,?,?,?,?,?,?,?,?,?,?,?,?,?,?)")
            .bind(&row.id).bind(&row.segment_id).bind(row.attempt_epoch).bind(&row.input_digest).bind(&row.model_spec_id)
            .bind(&row.engine_version).bind(row.seed).bind(json(&row.synthesis_config)?).bind(&row.state).bind(&row.asset_path)
            .bind(&row.output_sha256).bind(row.sample_rate).bind(row.sample_count).bind(row.created_at).bind(row.published_at)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    for row in &request.speech_projects {
        sqlx::query("UPDATE spaces_speech_projects SET active_revision_id=? WHERE id=?")
            .bind(&row.active_revision_id)
            .bind(&row.id)
            .execute(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    }
    Ok(())
}

fn json(value: &serde_json::Value) -> Result<String, String> {
    serde_json::to_string(value).map_err(db::json_error)
}
