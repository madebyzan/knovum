use sqlx::{Sqlite, Transaction};

use super::{AcceptedSourceSeparation, SpeechMediaSource, VideoDubbingOutputConfig};
use crate::spaces::db;

pub(super) fn validate(config: &VideoDubbingOutputConfig) -> Result<(), String> {
    match (&config.background, &config.background_separation) {
        (None, None) | (Some(_), None) => Ok(()),
        (Some(_), Some(separation))
            if uuid::Uuid::parse_str(&separation.separation_id).is_ok()
                && crate::local_resources::model_catalog::LocalModelId::from_separation_spec(
                    &separation.model_spec_id,
                )
                .is_ok()
                && separation.source == config.video =>
        {
            Ok(())
        }
        _ => Err("Video background separation provenance is invalid".into()),
    }
}

pub(super) async fn require_accepted(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    separation: &AcceptedSourceSeparation,
    background: &SpeechMediaSource,
) -> Result<(), String> {
    let accepted: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spaces_speech_source_separations WHERE id=? AND space_id=? \
         AND review_status='accepted' AND model_spec_id=? AND source_asset_path=? AND source_revision=? \
         AND source_duration_ms=? AND background_asset_path=? AND 'sha256:' || background_sha256=? \
         AND background_duration_ms=?)",
    )
    .bind(&separation.separation_id)
    .bind(space_id)
    .bind(&separation.model_spec_id)
    .bind(&separation.source.asset_path)
    .bind(&separation.source.source_revision)
    .bind(i64::try_from(separation.source.duration_ms).map_err(|_| "Media duration is too large")?)
    .bind(&background.asset_path)
    .bind(&background.source_revision)
    .bind(i64::try_from(background.duration_ms).map_err(|_| "Media duration is too large")?)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    if accepted {
        Ok(())
    } else {
        Err("Accepted background no longer matches its source separation revision".into())
    }
}
