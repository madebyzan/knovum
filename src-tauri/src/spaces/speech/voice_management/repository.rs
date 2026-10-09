use std::collections::HashMap;

use sqlx::Row;

use super::contracts::{VoiceProfileHistory, VoiceRevision};
use crate::spaces::{db, speech::contracts::parse_synthesis_config};

pub(super) async fn list(pool: &sqlx::SqlitePool) -> Result<Vec<VoiceProfileHistory>, String> {
    let rows = sqlx::query(
        "SELECT p.id,p.name,p.source_kind,p.active_revision_id,p.archived_at,p.updated_at, \
         (SELECT COUNT(*) FROM spaces_speech_segments s JOIN spaces_voice_revisions sr \
           ON sr.id=s.voice_revision_id WHERE sr.profile_id=p.id) + \
         (SELECT COUNT(*) FROM spaces_voice_revisions rr, \
           spaces_speech_project_revisions revision, json_each(revision.output_config,'$.roles') role \
           WHERE rr.profile_id=p.id AND json_extract(role.value,'$.voiceRevisionId')=rr.id) \
         AS referenced_binding_count \
         FROM spaces_voice_profiles p ORDER BY p.archived_at IS NOT NULL,p.updated_at DESC,p.id",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    let mut revisions = revisions_by_profile(pool).await?;
    let mut profiles = Vec::with_capacity(rows.len());
    for row in rows {
        let id: String = row.try_get("id").map_err(db::sql_error)?;
        let active_revision_id: String = row
            .try_get::<Option<String>, _>("active_revision_id")
            .map_err(db::sql_error)?
            .ok_or("Voice profile has no active revision")?;
        let profile_revisions = revisions
            .remove(&id)
            .filter(|items| {
                items
                    .iter()
                    .any(|revision| revision.id == active_revision_id)
            })
            .ok_or("Voice profile active revision is unavailable")?;
        profiles.push(VoiceProfileHistory {
            revisions: profile_revisions,
            id,
            name: row.try_get("name").map_err(db::sql_error)?,
            source_kind: row.try_get("source_kind").map_err(db::sql_error)?,
            active_revision_id,
            archived_at: row.try_get("archived_at").map_err(db::sql_error)?,
            updated_at: row.try_get("updated_at").map_err(db::sql_error)?,
            referenced_binding_count: row
                .try_get("referenced_binding_count")
                .map_err(db::sql_error)?,
        });
    }
    Ok(profiles)
}

async fn revisions_by_profile(
    pool: &sqlx::SqlitePool,
) -> Result<HashMap<String, Vec<VoiceRevision>>, String> {
    let rows = sqlx::query(
        "SELECT r.profile_id,r.id,r.revision,r.engine_id,r.model_spec_id,r.language,r.synthesis_config, \
         r.reference_source_revision,r.design_description,r.created_at FROM spaces_voice_revisions r \
         ORDER BY r.profile_id,r.revision DESC",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    let mut revisions = HashMap::<String, Vec<VoiceRevision>>::new();
    for row in rows {
        let stored_config: String = row.try_get("synthesis_config").map_err(db::sql_error)?;
        let config =
            parse_synthesis_config(&serde_json::from_str(&stored_config).map_err(db::json_error)?)?;
        revisions
            .entry(row.try_get("profile_id").map_err(db::sql_error)?)
            .or_default()
            .push(VoiceRevision {
                id: row.try_get("id").map_err(db::sql_error)?,
                revision: row.try_get("revision").map_err(db::sql_error)?,
                engine_id: row.try_get("engine_id").map_err(db::sql_error)?,
                model_spec_id: row.try_get("model_spec_id").map_err(db::sql_error)?,
                language: row.try_get("language").map_err(db::sql_error)?,
                speed: config.speed,
                sample_rate: config.sample_rate,
                reference_source_revision: row
                    .try_get("reference_source_revision")
                    .map_err(db::sql_error)?,
                design_description: row.try_get("design_description").map_err(db::sql_error)?,
                created_at: row.try_get("created_at").map_err(db::sql_error)?,
            });
    }
    Ok(revisions)
}
