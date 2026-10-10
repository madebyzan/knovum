use serde::Deserialize;
use serde_json::json;

use super::super::video_config::SpeechMediaSource;
use super::{
    contracts::{SpeechSourceSeparationPublication, StartSpeechSourceSeparationRequest},
    SeparationConfiguration, WORKFLOW,
};
use crate::{
    app_error::AppError,
    spaces::{
        authorization::{create_grant, local_user_grant, RunScope},
        db,
        runs::{create_run_in_transaction, NewRun, RunKind, RunObservation},
    },
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SeparationRunInput {
    source_node_id: String,
    vocals_label: String,
    background_label: String,
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
    for label in [&request.vocals_label, &request.background_label] {
        if label.trim().is_empty() || label.chars().count() > 80 {
            return Err("Source separation track labels must contain 1 to 80 characters".into());
        }
    }
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    if let Some((scope_kind, home_space_id, kind, input_json)) =
        sqlx::query_as::<_, (String, Option<String>, String, String)>(
            "SELECT scope_kind,home_space_id,kind,input_snapshot FROM spaces_runs WHERE id=?",
        )
        .bind(&request.request_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db::sql_error)?
    {
        let scope = RunScope::from_db(&scope_kind, home_space_id)?;
        let is_same_request = scope
            .require_space_id()
            .is_ok_and(|space_id| matches_existing_request(space_id, &kind, &input_json, &request));
        tx.rollback().await.map_err(db::sql_error)?;
        if !is_same_request {
            return Err(
                "request_conflict: request identity belongs to a different source separation"
                    .into(),
            );
        }
        return Ok(crate::spaces::runs::load_run(pool, &request.request_id).await?);
    }
    super::media::require_owned_source(
        &mut tx,
        &request.space_id,
        &request.source_node_id,
        &request.source,
    )
    .await?;
    let model = crate::local_resources::model_selection::speech_separation(&app)?;
    crate::ai::local_runtime::device_profiles::speech_separation(model)?;
    let now = crate::clock::unix_time_millis()?;
    let scope = RunScope::Space {
        space_id: request.space_id.clone(),
    };
    let grant = local_user_grant(
        uuid::Uuid::new_v4().to_string(),
        scope.clone(),
        Vec::new(),
        now,
    )?;
    let principal = grant.principal.clone();
    create_grant(&mut tx, &grant, now).await?;
    let input = json!({"workflow":WORKFLOW,"sourceNodeId":request.source_node_id,"vocalsLabel":request.vocals_label,"backgroundLabel":request.background_label,"source":request.source.clone(),"model":model});
    let policy = json!({"grantId":grant.grant_id,"definition":{"failurePolicy":"stop_run"},"workflow":WORKFLOW});
    let budget = json!({"maxParallelism":1,"maxAttempts":3,"maxStepAttempts":3});
    create_run_in_transaction(
        &mut tx,
        &NewRun {
            id: &request.request_id,
            scope: &scope,
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
    .bind(sqlx::types::Json(SeparationConfiguration {source_node_id:request.source_node_id, vocals_label:request.vocals_label, background_label:request.background_label, source:request.source, model}))
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
pub async fn list_source_separations(
    app: tauri::AppHandle,
    space_id: String,
    source: SpeechMediaSource,
) -> Result<Vec<SpeechSourceSeparationPublication>, AppError> {
    crate::spaces::assets::authorize_spaces_asset(&app, std::path::Path::new(&source.asset_path))?;
    Ok(
        super::repository::list_publications_for_source(db::pool(&app).await?, &space_id, &source)
            .await?,
    )
}

#[tauri::command]
pub async fn get_latest_source_separation_run(
    app: tauri::AppHandle,
    space_id: String,
    source_node_id: String,
    source: SpeechMediaSource,
) -> Result<Option<RunObservation>, AppError> {
    crate::spaces::assets::authorize_spaces_asset(&app, std::path::Path::new(&source.asset_path))?;
    let pool = db::pool(&app).await?;
    let run_id: Option<String> = sqlx::query_scalar("SELECT id FROM spaces_runs WHERE scope_kind='space' AND home_space_id=? AND json_extract(input_snapshot,'$.workflow')=? AND json_extract(input_snapshot,'$.sourceNodeId')=? AND json_extract(input_snapshot,'$.source.assetPath')=? AND json_extract(input_snapshot,'$.source.sourceRevision')=? AND json_extract(input_snapshot,'$.source.durationMs')=? ORDER BY created_at DESC,id DESC LIMIT 1")
        .bind(&space_id).bind(WORKFLOW).bind(&source_node_id).bind(&source.asset_path).bind(&source.source_revision)
        .bind(i64::try_from(source.duration_ms).map_err(|_| "Media duration is too large")?)
        .fetch_optional(pool).await.map_err(db::sql_error)?;
    match run_id {
        Some(id) => Ok(Some(crate::spaces::runs::load_run(pool, &id).await?)),
        None => Ok(None),
    }
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
            && input.source_node_id == request.source_node_id
            && input.vocals_label == request.vocals_label
            && input.background_label == request.background_label
            && input.source == request.source
            && input.model.separation_spec().is_ok()
    })
}
