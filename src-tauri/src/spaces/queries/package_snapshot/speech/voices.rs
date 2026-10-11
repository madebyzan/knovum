use serde_json::{json, Value};
use sqlx::{Sqlite, Transaction};

use super::super::{
    decode::{
        decode_integer, decode_json, decode_optional_integer, decode_optional_text, decode_text,
    },
    package_assets::PackageAssets,
};
use crate::spaces::db;

pub(super) async fn load(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    assets: &mut PackageAssets,
) -> Result<(Vec<Value>, Vec<Value>), String> {
    let used = sqlx::query(
        "SELECT DISTINCT v.id,v.profile_id FROM spaces_voice_revisions v WHERE v.id IN (
           SELECT s.voice_revision_id FROM spaces_speech_segments s
           JOIN spaces_speech_project_revisions r ON r.id=s.project_revision_id
           JOIN spaces_speech_projects p ON p.id=r.project_id
           JOIN spaces_nodes n ON n.id=p.source_node_id AND n.deleted_at IS NULL
           WHERE p.space_id=$1
           UNION
           SELECT json_extract(role.value,'$.voiceRevisionId')
           FROM spaces_speech_project_revisions r
           JOIN spaces_speech_projects p ON p.id=r.project_id
           JOIN spaces_nodes n ON n.id=p.source_node_id AND n.deleted_at IS NULL
           JOIN json_each(r.output_config,'$.roles') role
           WHERE p.space_id=$1
         ) ORDER BY v.profile_id,v.revision",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let profile_ids = used
        .iter()
        .map(|row| decode_text(row, "profile_id"))
        .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
    let profiles = sqlx::query(
        "SELECT * FROM spaces_voice_profiles WHERE id IN (SELECT value FROM json_each(?))
         ORDER BY created_at,id",
    )
    .bind(sqlx::types::Json(profile_ids))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let revision_ids = used
        .iter()
        .map(|row| decode_text(row, "id"))
        .chain(
            profiles
                .iter()
                .map(|row| decode_text(row, "active_revision_id")),
        )
        .collect::<Result<std::collections::BTreeSet<_>, _>>()?;
    let revisions = sqlx::query(
        "SELECT * FROM spaces_voice_revisions WHERE id IN (SELECT value FROM json_each(?))
         ORDER BY profile_id,revision",
    )
    .bind(sqlx::types::Json(revision_ids))
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    Ok((
        profiles.iter().map(profile).collect::<Result<_, _>>()?,
        revisions
            .iter()
            .map(|row| voice_revision(row, assets))
            .collect::<Result<_, _>>()?,
    ))
}

fn profile(row: &sqlx::sqlite::SqliteRow) -> Result<Value, String> {
    Ok(json!({
        "id":decode_text(row,"id")?, "name":decode_text(row,"name")?,
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
