use serde::Deserialize;
use serde_json::json;

use super::super::video_config::SpeechMediaSource;
use super::{
    contracts::{
        ReviewSpeechSourceSeparationRequest, SpeechSourceSeparationCandidate,
        SpeechSourceSeparationReviewStatus, StartSpeechSourceSeparationRequest,
    },
    repository::{load_candidate, transition_review},
    WORKFLOW,
};
use crate::{
    app_error::AppError,
    spaces::{
        authorization::{create_grant, AuthorizationContext, Principal, PrincipalKind},
        db,
        runs::{create_run_in_transaction, NewRun, RunKind, RunObservation},
    },
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SeparationRunInput {
    workflow: String,
    source: SpeechMediaSource,
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
    crate::ai::local_runtime::device_profiles::speech_separation()?;
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
    super::super::video_config::require_source(
        &mut tx,
        &request.space_id,
        None,
        "video",
        &request.source,
    )
    .await?;
    let now = crate::clock::unix_time_millis()?;
    let principal = Principal {
        kind: PrincipalKind::User,
        id: "local-user".into(),
    };
    let grant = AuthorizationContext {
        principal: principal.clone(),
        grant_id: uuid::Uuid::new_v4().to_string(),
        home_space_id: request.space_id.clone(),
        read_space_ids: vec![request.space_id.clone()],
        write_space_ids: vec![request.space_id.clone()],
        action_ids: Vec::new(),
        valid_until: now + 7 * 24 * 60 * 60 * 1_000,
    };
    create_grant(&mut tx, &grant, now).await?;
    let input = json!({"workflow":WORKFLOW,"source":request.source.clone()});
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
    .bind(serde_json::to_string(&json!({"source":request.source})).map_err(db::json_error)?)
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
    let path = crate::spaces::assets::authorize_spaces_asset(
        &app,
        std::path::Path::new(&current.background.asset_path),
    )?;
    let actual_sha256: String =
        crate::blocking::run(move || super::super::publication::hash_file(&path)).await?;
    if actual_sha256
        != current
            .background
            .source_revision
            .strip_prefix("sha256:")
            .ok_or("Separated background revision is invalid")?
    {
        return Err("stale_revision: separated background changed before review".into());
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
    })
}
