use serde_json::json;
use sha2::{Digest, Sha256};

use super::{
    contracts::{TranslateVideoDubbingRequest, TranslationRunInput, TranslationStepConfiguration},
    repository, WORKFLOW,
};
use crate::{
    ai::{contracts::ExecutionCapability, settings},
    app_error::AppError,
    spaces::{
        authorization::{create_grant, AuthorizationContext, Principal, PrincipalKind},
        db,
        runs::{create_run_in_transaction, NewRun, RunKind, RunObservation},
    },
};

#[tauri::command]
pub async fn translate_video_dubbing(
    app: tauri::AppHandle,
    request: TranslateVideoDubbingRequest,
) -> Result<RunObservation, AppError> {
    repository::validate_request(&request)?;
    let mut target = settings::resolve(
        &app,
        request.execution_target.clone(),
        ExecutionCapability::TextGeneration,
    )?;
    target.temperature = 0.0;
    if request.local_only && !target.is_local() {
        return Err(AppError::invalid(
            "Local-only video dubbing requires an Ollama model on a loopback endpoint",
        ));
    }
    let pool = db::pool(&app).await?;
    let mut tx = db::begin_write(pool).await?;
    let project = repository::active_project(&mut tx, &request.project_id).await?;
    repository::require_expected_revision(&project, request.expected_revision)?;
    repository::require_current_segments(
        &mut tx,
        &project.revision_id,
        project.translated,
        &request.segments,
    )
    .await?;
    let input_revision = input_revision(
        &request,
        &project.revision_id,
        target.provider(),
        target.model(),
        &target.base_url,
    )?;
    let input = TranslationRunInput {
        workflow: WORKFLOW.into(),
        project_id: request.project_id.clone(),
        revision_id: project.revision_id.clone(),
        expected_revision: request.expected_revision,
        input_revision,
        target_language: request.target_language.clone(),
        local_only: request.local_only,
        provider: target.provider().into(),
        model: target.model().into(),
        base_url: target.base_url.clone(),
        execution_target: request.execution_target.clone(),
        segments: request.segments.clone(),
    };
    if let Some(existing) = repository::existing_request(&mut tx, &request.job_id).await? {
        let same = existing == input;
        tx.rollback().await.map_err(db::sql_error)?;
        if !same {
            return Err(
                "request_conflict: translation request identity belongs to another Run".into(),
            );
        }
        return Ok(crate::spaces::runs::load_run(pool, &request.job_id).await?);
    }
    if let Some((run_id, existing)) =
        repository::active_for_revision(&mut tx, &request.project_id, request.expected_revision)
            .await?
    {
        let same = existing == input;
        tx.rollback().await.map_err(db::sql_error)?;
        if same {
            return Ok(crate::spaces::runs::load_run(pool, &run_id).await?);
        }
        return Err("resource_busy: video translation already has an active Run".into());
    }
    create_run(&mut tx, &request.job_id, &project.space_id, &input).await?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    Ok(crate::spaces::runs::load_run(pool, &request.job_id).await?)
}

#[tauri::command]
pub async fn get_video_translation_run(
    app: tauri::AppHandle,
    project_id: String,
    expected_revision: i64,
) -> Result<Option<RunObservation>, AppError> {
    let pool = db::pool(&app).await?;
    let run_id = repository::latest_recoverable_run(pool, &project_id, expected_revision).await?;
    match run_id {
        Some(run_id) => Ok(Some(crate::spaces::runs::load_run(pool, &run_id).await?)),
        None => Ok(None),
    }
}

#[tauri::command]
pub async fn get_video_translation(
    app: tauri::AppHandle,
    run_id: String,
) -> Result<super::VideoTranslation, AppError> {
    Ok(repository::load_translation(db::pool(&app).await?, &run_id).await?)
}

fn input_revision(
    request: &TranslateVideoDubbingRequest,
    revision_id: &str,
    provider: &str,
    model: &str,
    base_url: &str,
) -> Result<String, String> {
    let bytes = serde_json::to_vec(&json!({
        "projectId": request.project_id,
        "revisionId": revision_id,
        "expectedRevision": request.expected_revision,
        "targetLanguage": request.target_language,
        "localOnly": request.local_only,
        "provider": provider,
        "model": model,
        "baseUrl": base_url,
        "segments": request.segments,
    }))
    .map_err(db::json_error)?;
    Ok(format!("sha256:{}", hex::encode(Sha256::digest(bytes))))
}

async fn create_run(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    run_id: &str,
    space_id: &str,
    input: &TranslationRunInput,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let principal = Principal {
        kind: PrincipalKind::User,
        id: "local-user".into(),
    };
    let grant = AuthorizationContext {
        principal: principal.clone(),
        grant_id: uuid::Uuid::new_v4().to_string(),
        home_space_id: space_id.into(),
        read_space_ids: vec![space_id.into()],
        write_space_ids: vec![space_id.into()],
        action_ids: Vec::new(),
        valid_until: now + 7 * 24 * 60 * 60 * 1_000,
    };
    create_grant(tx, &grant, now).await?;
    let input_json = serde_json::to_value(input).map_err(db::json_error)?;
    let policy = json!({
        "grantId": grant.grant_id,
        "definition": {"failurePolicy": "stop_run"},
        "workflow": WORKFLOW,
        "provider": input.provider,
        "model": input.model,
        "localOnly": input.local_only,
    });
    let step_attempts = i64::try_from(input.segments.len())
        .map_err(|_| "Video translation has too many segments")?
        .checked_mul(3)
        .ok_or("Video translation attempt budget overflow")?;
    let budget = json!({"maxParallelism":1,"maxAttempts":3,"maxStepAttempts":step_attempts});
    create_run_in_transaction(
        tx,
        &NewRun {
            id: run_id,
            home_space_id: space_id,
            parent_run_id: None,
            principal: &principal,
            kind: RunKind::Workflow,
            policy_snapshot: &policy,
            input_snapshot: &input_json,
            budget: &budget,
        },
        now,
    )
    .await?;
    for (ordinal, segment) in input.segments.iter().enumerate() {
        let configuration = TranslationStepConfiguration {
            project_id: input.project_id.clone(),
            revision_id: input.revision_id.clone(),
            expected_revision: input.expected_revision,
            input_revision: input.input_revision.clone(),
            target_language: input.target_language.clone(),
            local_only: input.local_only,
            provider: input.provider.clone(),
            model: input.model.clone(),
            base_url: input.base_url.clone(),
            execution_target: input.execution_target.clone(),
            ordinal,
            segment: segment.clone(),
        };
        sqlx::query(
            "INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,depends_on,configuration,state,state_version,resolved_input,created_at,updated_at) \
             VALUES(?,?,?,?,?,'[]',?,'ready',0,'{}',?,?)",
        )
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(run_id)
        .bind(format!("translation:{ordinal}"))
        .bind(format!("translation:{ordinal}"))
        .bind(super::NODE_KIND)
        .bind(serde_json::to_string(&configuration).map_err(db::json_error)?)
        .bind(now)
        .bind(now)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    }
    sqlx::query("UPDATE spaces_runs SET expanded_steps=? WHERE id=?")
        .bind(
            i64::try_from(input.segments.len())
                .map_err(|_| "Video translation step count overflow")?,
        )
        .bind(run_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    Ok(())
}
