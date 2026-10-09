use std::collections::BTreeSet;

use super::contracts::DeleteVoiceProfileRequest;
use crate::spaces::db;

pub(super) async fn execute(
    app: &tauri::AppHandle,
    request: &DeleteVoiceProfileRequest,
) -> Result<(), String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let candidate_assets: BTreeSet<String> = sqlx::query_scalar(
        "SELECT r.reference_asset_id FROM spaces_voice_revisions r JOIN spaces_voice_profiles p \
         ON p.id=r.profile_id WHERE p.id=? AND p.updated_at=? AND p.archived_at IS NOT NULL \
         AND r.reference_asset_id IS NOT NULL \
         UNION ALL SELECT candidate.reference_asset_path FROM spaces_voice_candidates candidate \
         WHERE candidate.selected_voice_profile_id=? \
            OR candidate.selected_voice_revision_id IN ( \
              SELECT id FROM spaces_voice_revisions WHERE profile_id=?) \
         UNION ALL SELECT candidate.preview_asset_path FROM spaces_voice_candidates candidate \
         WHERE candidate.selected_voice_profile_id=? \
            OR candidate.selected_voice_revision_id IN ( \
              SELECT id FROM spaces_voice_revisions WHERE profile_id=?) \
         UNION ALL SELECT json_extract(run.input_snapshot,'$.referenceSource.assetPath') \
         FROM spaces_runs run JOIN spaces_voice_candidates candidate ON candidate.run_id=run.id \
         WHERE (candidate.selected_voice_profile_id=? \
            OR candidate.selected_voice_revision_id IN ( \
              SELECT id FROM spaces_voice_revisions WHERE profile_id=?)) \
           AND json_type(run.input_snapshot,'$.referenceSource.assetPath')='text'",
    )
    .bind(&request.profile_id)
    .bind(request.expected_updated_at)
    .bind(&request.profile_id)
    .bind(&request.profile_id)
    .bind(&request.profile_id)
    .bind(&request.profile_id)
    .bind(&request.profile_id)
    .bind(&request.profile_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(db::sql_error)?
    .into_iter()
    .collect();
    let grant_ids: Vec<String> = sqlx::query_scalar(
        "DELETE FROM spaces_runs WHERE scope_kind='application' AND state='succeeded' \
         AND completed_at IS NOT NULL \
         AND json_extract(input_snapshot,'$.workflow')='speech_voice_candidate_v1' \
         AND id IN (SELECT candidate.run_id FROM spaces_voice_candidates candidate \
           WHERE candidate.status='selected' AND (candidate.selected_voice_profile_id=? \
             OR candidate.selected_voice_revision_id IN ( \
               SELECT id FROM spaces_voice_revisions WHERE profile_id=?))) \
         RETURNING json_extract(policy_snapshot,'$.grantId')",
    )
    .bind(&request.profile_id)
    .bind(&request.profile_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    crate::spaces::authorization::delete_unreferenced_application_grants(&mut tx, &grant_ids)
        .await?;
    let deleted = sqlx::query(
        "DELETE FROM spaces_voice_profiles WHERE id=? AND updated_at=? AND archived_at IS NOT NULL \
         AND NOT EXISTS(SELECT 1 FROM spaces_speech_segments s JOIN spaces_voice_revisions r \
           ON r.id=s.voice_revision_id WHERE r.profile_id=spaces_voice_profiles.id) \
         AND NOT EXISTS(SELECT 1 FROM spaces_voice_revisions r, \
           spaces_speech_project_revisions revision, json_each(revision.output_config,'$.roles') role \
           WHERE r.profile_id=spaces_voice_profiles.id \
             AND json_extract(role.value,'$.voiceRevisionId')=r.id) \
         AND NOT EXISTS(SELECT 1 FROM spaces_voice_candidates candidate \
           WHERE candidate.selected_voice_profile_id=spaces_voice_profiles.id \
              OR candidate.selected_voice_revision_id IN ( \
                SELECT id FROM spaces_voice_revisions \
                WHERE profile_id=spaces_voice_profiles.id))",
    )
    .bind(&request.profile_id)
    .bind(request.expected_updated_at)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    db::require_one(
        deleted,
        "Only an archived, unreferenced voice profile can be permanently deleted",
    )?;
    tx.commit().await.map_err(db::sql_error)?;
    super::super::voice_assets::reclaim_unreferenced(app, pool, candidate_assets)
        .await
        .map_err(|error| {
            format!("Voice profile was deleted, but its asset cleanup failed: {error}")
        })?;
    Ok(())
}
