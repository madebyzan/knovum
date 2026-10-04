use std::{collections::BTreeSet, fs, path::Path};

use super::contracts::DeleteVoiceProfileRequest;
use crate::spaces::{assets, db};

pub(super) async fn execute(
    app: &tauri::AppHandle,
    request: &DeleteVoiceProfileRequest,
) -> Result<(), String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let candidates: BTreeSet<String> = sqlx::query_scalar(
        "SELECT r.reference_asset_id FROM spaces_voice_revisions r JOIN spaces_voice_profiles p \
         ON p.id=r.profile_id WHERE p.id=? AND p.updated_at=? AND p.archived_at IS NOT NULL \
         AND r.reference_asset_id IS NOT NULL",
    )
    .bind(&request.profile_id)
    .bind(request.expected_updated_at)
    .fetch_all(&mut *tx)
    .await
    .map_err(db::sql_error)?
    .into_iter()
    .collect();
    let deleted = sqlx::query(
        "DELETE FROM spaces_voice_profiles WHERE id=? AND updated_at=? AND archived_at IS NOT NULL \
         AND NOT EXISTS(SELECT 1 FROM spaces_speech_segments s JOIN spaces_voice_revisions r \
           ON r.id=s.voice_revision_id WHERE r.profile_id=spaces_voice_profiles.id) \
         AND NOT EXISTS(SELECT 1 FROM spaces_voice_revisions r \
           JOIN spaces_speech_projects project ON project.space_id=spaces_voice_profiles.space_id \
           JOIN spaces_speech_project_revisions revision ON revision.project_id=project.id \
           JOIN json_each(revision.output_config,'$.roles') role \
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
    let removable = match assets::unreferenced_paths(pool, candidates).await {
        Ok(paths) => paths,
        Err(error) => {
            log::warn!("Deleted voice asset reference check failed: {error}");
            return Ok(());
        }
    };
    for path in removable {
        match assets::authorize_spaces_asset(app, Path::new(&path)) {
            Ok(authorized) => {
                if let Err(error) = fs::remove_file(authorized) {
                    log::warn!("Deleted voice asset cleanup failed: {error}");
                }
            }
            Err(error) => log::warn!("Deleted voice asset was outside managed storage: {error}"),
        }
    }
    Ok(())
}
