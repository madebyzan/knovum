use serde::Deserialize;
use serde_json::json;

use super::super::video_config::SpeechMediaSource;
use super::{
    contracts::{
        ReviewSpeechSourceSeparationRequest, SpeechSourceSeparationCandidate,
        SpeechSourceSeparationReviewStatus, StartSpeechSourceSeparationRequest,
    },
    repository::{load_candidate, transition_review},
    SeparationConfiguration, WORKFLOW,
};
use crate::{
    app_error::AppError,
    spaces::{
        authorization::{create_grant, local_user_grant},
        db,
        runs::{create_run_in_transaction, NewRun, RunKind, RunObservation},
    },
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SeparationRunInput {
    workflow: String,
    source: SpeechMediaSource,
    model: crate::local_resources::model_catalog::LocalModelId,
}

#[tauri::command]
pub async fn start_speech_source_separation(
    app: tauri::AppHandle,
    request: StartSpeechSourceSeparationRequest,
) -> Result<RunObservation, AppError> {
    require_canonical_uuid(&request.request_id, "Source separation request identity")?;
    if request.source.duration_ms == 0 || request.source.duration_ms > 600_000 {
        return Err("Source separation requires media lasting no more than ten minutes".into());
    }
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    if let Some((space_id, kind, input_json)) = sqlx::query_as::<_, (String, String, String)>(
        "SELECT home_space_id,kind,input_snapshot FROM spaces_runs WHERE id=?",
    )
    .bind(&request.request_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?
    {
        let is_same_request = matches_existing_request(&space_id, &kind, &input_json, &request);
        tx.rollback().await.map_err(db::sql_error)?;
        if !is_same_request {
            return Err(
                "request_conflict: request identity belongs to a different source separation"
                    .into(),
            );
        }
        return Ok(crate::spaces::runs::load_run(pool, &request.request_id).await?);
    }
    super::media::require_owned_source(&mut tx, &request.space_id, &request.source).await?;
    let model = crate::local_resources::model_selection::speech_separation(&app)?;
    crate::ai::local_runtime::device_profiles::speech_separation(model)?;
    let now = crate::clock::unix_time_millis()?;
    let grant = local_user_grant(
        uuid::Uuid::new_v4().to_string(),
        request.space_id.clone(),
        Vec::new(),
        now,
    );
    let principal = grant.principal.clone();
    create_grant(&mut tx, &grant, now).await?;
    let input = json!({"workflow":WORKFLOW,"source":request.source.clone(),"model":model});
    let policy = json!({"grantId":grant.grant_id,"definition":{"failurePolicy":"stop_run"},"workflow":WORKFLOW});
    let budget = json!({"maxParallelism":1,"maxAttempts":3,"maxStepAttempts":3});
    create_run_in_transaction(
        &mut tx,
        &NewRun {
            id: &request.request_id,
            home_space_id: &request.space_id,
            parent_run_id: None,
            principal: &principal,
            kind: RunKind::Workflow,
            policy_snapshot: &policy,
            input_snapshot: &input,
            budget: &budget,
        },
        now,
    )
    .await?;
    sqlx::query(
        "INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,depends_on,configuration,state,state_version,resolved_input,created_at,updated_at) \
         VALUES(?,?,'separate','separate','speech_source_separation','[]',?,'ready',0,'{}',?,?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&request.request_id)
    .bind(sqlx::types::Json(SeparationConfiguration {source:request.source, model}))
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_runs SET expanded_steps=1 WHERE id=?")
        .bind(&request.request_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(crate::spaces::runs::load_run(pool, &request.request_id).await?)
}

#[tauri::command]
pub async fn get_speech_source_separation(
    app: tauri::AppHandle,
    separation_id: String,
) -> Result<SpeechSourceSeparationCandidate, AppError> {
    require_canonical_uuid(&separation_id, "Source separation identity")?;
    Ok(load_candidate(db::pool(&app).await?, &separation_id).await?)
}

#[tauri::command]
pub async fn list_accepted_source_separations(
    app: tauri::AppHandle,
    space_id: String,
    source: SpeechMediaSource,
) -> Result<Vec<SpeechSourceSeparationCandidate>, AppError> {
    crate::spaces::assets::authorize_spaces_asset(&app, std::path::Path::new(&source.asset_path))?;
    Ok(super::repository::accepted_for_source(db::pool(&app).await?, &space_id, &source).await?)
}

#[tauri::command]
pub async fn get_latest_source_separation_run(
    app: tauri::AppHandle,
    space_id: String,
    source: SpeechMediaSource,
) -> Result<Option<RunObservation>, AppError> {
    crate::spaces::assets::authorize_spaces_asset(&app, std::path::Path::new(&source.asset_path))?;
    let pool = db::pool(&app).await?;
    let run_id: Option<String> = sqlx::query_scalar("SELECT id FROM spaces_runs WHERE home_space_id=? AND json_extract(input_snapshot,'$.workflow')=? AND json_extract(input_snapshot,'$.source.assetPath')=? AND json_extract(input_snapshot,'$.source.sourceRevision')=? ORDER BY created_at DESC,id DESC LIMIT 1")
        .bind(&space_id).bind(WORKFLOW).bind(&source.asset_path).bind(&source.source_revision)
        .fetch_optional(pool).await.map_err(db::sql_error)?;
    match run_id {
        Some(id) => Ok(Some(crate::spaces::runs::load_run(pool, &id).await?)),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn review_speech_source_separation(
    app: tauri::AppHandle,
    request: ReviewSpeechSourceSeparationRequest,
) -> Result<SpeechSourceSeparationCandidate, AppError> {
    require_canonical_uuid(&request.separation_id, "Source separation identity")?;
    if request.expected_review_status != SpeechSourceSeparationReviewStatus::Pending {
        return Err("Source separation review must target the pending revision".into());
    }
    let pool = db::pool(&app).await?;
    let current = load_candidate(pool, &request.separation_id).await?;
    for stem in [&current.background, &current.vocals] {
        let path = crate::spaces::assets::authorize_spaces_asset(
            &app,
            std::path::Path::new(&stem.asset_path),
        )?;
        let expected = stem
            .source_revision
            .strip_prefix("sha256:")
            .ok_or("Separated track revision is invalid")?;
        let actual_sha256: String =
            crate::blocking::run(move || super::super::publication::hash_file(&path)).await?;
        if actual_sha256 != expected {
            return Err("stale_revision: separated track changed before review".into());
        }
    }
    transition_review(
        pool,
        &request.separation_id,
        request.expected_review_status,
        request.accepted,
        crate::clock::unix_time_millis()?,
    )
    .await?;
    Ok(load_candidate(pool, &request.separation_id).await?)
}

fn require_canonical_uuid(value: &str, label: &str) -> Result<(), String> {
    let parsed = uuid::Uuid::parse_str(value).map_err(|_| format!("{label} must be a UUID"))?;
    if parsed.to_string() != value {
        return Err(format!("{label} must use canonical UUID form"));
    }
    Ok(())
}

pub(super) fn matches_existing_request(
    space_id: &str,
    kind: &str,
    input_json: &str,
    request: &StartSpeechSourceSeparationRequest,
) -> bool {
    serde_json::from_str::<SeparationRunInput>(input_json).is_ok_and(|input| {
        kind == "workflow"
            && space_id == request.space_id
            && input.workflow == WORKFLOW
            && input.source == request.source
            && input.model.separation_spec().is_ok()
    })
}
