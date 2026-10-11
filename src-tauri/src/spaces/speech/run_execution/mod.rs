mod assembly;
mod audiobook;
mod segment;

use crate::{
    foundation::app_error::AppError,
    spaces::{
        db,
        runs::{AttemptClaim, StepOutcome},
    },
};

pub(crate) async fn execute(
    app: &tauri::AppHandle,
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
) -> Result<StepOutcome, AppError> {
    require_authority(pool, claim).await?;
    match claim.node_kind.as_str() {
        "speech_segment" => segment::execute(app, pool, claim).await,
        "speech_chapter_pcm" => audiobook::execute_chapter(app, pool, claim).await,
        "speech_loudness_measurement" => audiobook::execute_loudness(app, pool, claim).await,
        "speech_audiobook_encode" => audiobook::execute_audiobook(app, pool, claim).await,
        "speech_assembly" => assembly::execute(app, pool, claim).await,
        _ => Err(AppError::new(
            "workflow_step_invalid",
            "Unsupported speech Run step",
        )),
    }
}

pub(super) async fn require_authority(
    pool: &sqlx::SqlitePool,
    claim: &AttemptClaim,
) -> Result<(), String> {
    let space_id = claim.scope.require_space_id()?;
    let grant_id = claim
        .policy_snapshot
        .get("grantId")
        .and_then(serde_json::Value::as_str)
        .ok_or("permission-denied: Speech Run authorization grant is missing")?;
    let mut tx = pool.begin().await.map_err(db::sql_error)?;
    let grant = crate::spaces::authorization::load_grant(&mut tx, grant_id).await?;
    grant.require_read(space_id, crate::foundation::clock::unix_time_millis()?)?;
    if grant.scope != claim.scope
        || !grant
            .write_space_ids
            .iter()
            .any(|candidate| candidate == space_id)
    {
        return Err("permission-denied: Speech Run requires Space write scope".into());
    }
    Ok(())
}
