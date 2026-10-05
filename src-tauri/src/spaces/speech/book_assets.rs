use sqlx::{Sqlite, Transaction};

use super::project_config::AudiobookCover;
use crate::spaces::db;

pub(super) async fn require_cover(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    project_id: Option<&str>,
    cover: Option<&AudiobookCover>,
) -> Result<(), String> {
    let Some(cover) = cover else {
        return Ok(());
    };
    let owned: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spaces_nodes WHERE space_id=? AND kind='image' AND deleted_at IS NULL AND json_extract(data,'$.assetPath')=? AND json_extract(data,'$.sourceRevision')=?)")
        .bind(space_id).bind(&cover.asset_path).bind(&cover.source_revision)
        .fetch_one(&mut **tx).await.map_err(db::sql_error)?;
    if owned {
        return Ok(());
    }
    if let Some(project_id) = project_id {
        let historical: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spaces_speech_project_revisions r JOIN spaces_speech_projects p ON p.id=r.project_id WHERE p.id=? AND p.space_id=? AND p.project_type='audiobook' AND json_extract(r.output_config,'$.metadata.cover.assetPath')=? AND json_extract(r.output_config,'$.metadata.cover.sourceRevision')=?)")
            .bind(project_id).bind(space_id).bind(&cover.asset_path).bind(&cover.source_revision)
            .fetch_one(&mut **tx).await.map_err(db::sql_error)?;
        if historical {
            return Ok(());
        }
    }
    Err(
        "Audiobook cover is not owned by this Space or an immutable revision of this project"
            .into(),
    )
}
