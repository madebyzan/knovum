use crate::spaces::speech::asset_publication::PublicationOwnerKind;

use std::{fs, path::Path};

use sqlx::FromRow;

use super::contracts::CopyVoiceProfileRequest;
use crate::spaces::{assets, db};

#[derive(FromRow)]
struct VoiceCopySource {
    space_id: String,
    source_kind: String,
    engine_id: String,
    model_spec_id: String,
    language: String,
    preset_voice_id: Option<String>,
    reference_asset_id: Option<String>,
    reference_source_revision: Option<String>,
    reference_text: Option<String>,
    design_description: Option<String>,
    synthesis_config: String,
    authorization: String,
}

pub(super) async fn execute(
    app: &tauri::AppHandle,
    request: &CopyVoiceProfileRequest,
) -> Result<String, String> {
    super::validate_name(&request.name)?;
    let pool = db::pool(app).await?;
    let source: VoiceCopySource = sqlx::query_as(
        "SELECT p.space_id,p.source_kind,r.engine_id,r.model_spec_id, \
         r.language,r.preset_voice_id,r.reference_asset_id,r.reference_source_revision,r.reference_text, \
         r.design_description,r.synthesis_config,r.authorization FROM spaces_voice_profiles p \
         JOIN spaces_voice_revisions r ON r.id=p.active_revision_id \
         WHERE p.id=? AND p.active_revision_id=?",
    )
    .bind(&request.profile_id)
    .bind(&request.expected_active_revision_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("Voice profile no longer exists")?;
    if source.space_id == request.target_space_id {
        return Err("Select another Space for the copied voice".into());
    }
    let target_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spaces WHERE id=? AND archived_at IS NULL)",
    )
    .bind(&request.target_space_id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    if !target_exists {
        return Err("Target Space is unavailable".into());
    }
    let profile_id = uuid::Uuid::new_v4().to_string();
    let copied = copy_reference(app, pool, &source, &profile_id, &request.target_space_id).await?;
    let result = persist(pool, request, &source, &profile_id, copied.as_ref()).await;
    match result {
        Ok(()) => Ok(profile_id),
        Err(error) => {
            if let Some((publication_id, _)) = copied {
                super::super::asset_publication::abandon(app, pool, &publication_id)
                    .await
                    .map_err(|cleanup| {
                        format!("{error}; copied reference cleanup failed: {cleanup}")
                    })?;
            }
            Err(error)
        }
    }
}

async fn copy_reference(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    source: &VoiceCopySource,
    profile_id: &str,
    target_space_id: &str,
) -> Result<Option<(String, String)>, String> {
    let Some(source_path) = source.reference_asset_id.as_deref() else {
        return Ok(None);
    };
    let revision = source
        .reference_source_revision
        .as_deref()
        .and_then(|value| value.strip_prefix("sha256:"))
        .ok_or("Voice reference revision is invalid")?;
    let source_path = assets::authorize_spaces_asset(app, Path::new(source_path))?;
    if super::super::publication::hash_file(&source_path)? != revision {
        return Err("Voice reference changed after its immutable revision was created".into());
    }
    super::super::voice_assets::reference_metadata(&source_path)?;
    let directory = assets::asset_directory(app)?.join("voices");
    fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    let target = directory.join(format!("{profile_id}.wav"));
    let part = directory.join(format!("{profile_id}.wav.part"));
    let publication_id = super::super::asset_publication::stage_unowned(
        pool,
        target_space_id,
        super::super::asset_publication::PendingAsset {
            owner_kind: PublicationOwnerKind::VoiceReference,
            owner_id: profile_id,
            asset_path: &target,
            part_path: Some(&part),
            sha256: revision,
        },
    )
    .await?;
    let copy_result = (|| {
        let mut input = fs::File::open(&source_path).map_err(|error| error.to_string())?;
        let mut output = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&part)
            .map_err(|error| error.to_string())?;
        std::io::copy(&mut input, &mut output).map_err(|error| error.to_string())?;
        output.sync_all().map_err(|error| error.to_string())?;
        drop(output);
        if super::super::publication::hash_file(&part)? != revision {
            return Err("Voice reference digest changed while copying".into());
        }
        fs::rename(&part, &target).map_err(|error| error.to_string())?;
        Ok(target.to_string_lossy().into_owned())
    })();
    match copy_result {
        Ok(path) => Ok(Some((publication_id, path))),
        Err(error) => {
            super::super::asset_publication::abandon(app, pool, &publication_id).await?;
            Err(error)
        }
    }
}

async fn persist(
    pool: &sqlx::SqlitePool,
    request: &CopyVoiceProfileRequest,
    source: &VoiceCopySource,
    profile_id: &str,
    copied: Option<&(String, String)>,
) -> Result<(), String> {
    let revision_id = uuid::Uuid::new_v4().to_string();
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let target_available: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spaces WHERE id=? AND archived_at IS NULL)",
    )
    .bind(&request.target_space_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if !target_available {
        return Err("Target Space changed while the voice was being copied".into());
    }
    let current: Option<String> = sqlx::query_scalar(
        "SELECT active_revision_id FROM spaces_voice_profiles WHERE id=? AND space_id=?",
    )
    .bind(&request.profile_id)
    .bind(&source.space_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if current.as_deref() != Some(request.expected_active_revision_id.as_str()) {
        return Err("Voice changed while it was being copied".into());
    }
    sqlx::query("INSERT INTO spaces_voice_profiles(id,space_id,name,source_kind,active_revision_id,archived_at,created_at,updated_at) VALUES(?,?,?,?,NULL,NULL,?,?)")
        .bind(profile_id).bind(&request.target_space_id).bind(request.name.trim()).bind(&source.source_kind)
        .bind(now).bind(now).execute(&mut *tx).await.map_err(db::sql_error)?;
    sqlx::query("INSERT INTO spaces_voice_revisions(id,profile_id,revision,engine_id,model_spec_id,language,preset_voice_id,reference_asset_id,reference_source_revision,reference_text,design_description,synthesis_config,authorization,created_at) VALUES(?,?,1,?,?,?,?,?,?,?,?,?,?,?)")
        .bind(&revision_id).bind(profile_id).bind(&source.engine_id).bind(&source.model_spec_id).bind(&source.language)
        .bind(&source.preset_voice_id).bind(copied.map(|(_, path)| path)).bind(&source.reference_source_revision)
        .bind(&source.reference_text).bind(&source.design_description).bind(&source.synthesis_config).bind(&source.authorization)
        .bind(now).execute(&mut *tx).await.map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_voice_profiles SET active_revision_id=? WHERE id=?")
        .bind(&revision_id)
        .bind(profile_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    if let Some((publication_id, _)) = copied {
        super::super::asset_publication::remove_in_transaction(&mut tx, publication_id).await?;
    }
    tx.commit().await.map_err(db::sql_error)
}
