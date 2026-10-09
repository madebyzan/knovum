use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};

use super::{
    DraftVoiceCandidateRequest, ReferenceSourceSnapshot, VoiceCandidateSource,
    VoiceCandidateStepConfiguration, WORKFLOW,
};
use crate::spaces::{
    authorization::{create_grant, local_user_grant, RunScope},
    db,
    runs::{create_run_in_transaction, NewRun, RunKind, RunObservation},
};

pub(super) async fn start(
    app: &tauri::AppHandle,
    request: DraftVoiceCandidateRequest,
) -> Result<RunObservation, String> {
    super::require_uuid(&request.request_id, "Voice candidate request identity")?;
    validate_request(&request)?;
    let (cancellation, _lease) = super::super::cancellation::acquire(&request.request_id)?;
    let incoming_digest = request_digest(&request)?;
    if matches!(
        request.model,
        super::VoiceModel::VoxCpm2 | super::VoiceModel::VoxCpm2Q8
    ) {
        crate::ai::local_runtime::device_profiles::voice::require_memory(
            request.model.specification(),
        )?;
    }
    let pool = db::pool(app).await?;
    if let Some((scope_kind, home_space_id, kind, input)) =
        sqlx::query_as::<_, (String, Option<String>, String, String)>(
            "SELECT scope_kind,home_space_id,kind,input_snapshot FROM spaces_runs WHERE id=?",
        )
        .bind(&request.request_id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)?
    {
        let scope = RunScope::from_db(&scope_kind, home_space_id)?;
        let matches = kind == "workflow"
            && scope.require_application().is_ok()
            && serde_json::from_str::<VoiceCandidateRunInput>(&input).is_ok_and(|stored| {
                stored.workflow == WORKFLOW
                    && stored.request_digest == incoming_digest
                    && stored.reference_source.is_some()
                        == matches!(&request.source, VoiceCandidateSource::Reference { .. })
            });
        if !matches {
            return Err(
                "request_conflict: request identity belongs to another voice candidate".into(),
            );
        }
        return crate::spaces::runs::load_run(pool, &request.request_id).await;
    }
    let frozen =
        super::source_snapshot::freeze(app, pool, request.clone(), incoming_digest, &cancellation)
            .await?;
    let creation = create(pool, &request, &frozen, &cancellation).await;
    if let Err(error) = creation {
        if let Some(publication_id) = frozen.publication_id {
            super::super::asset_publication::abandon(app, pool, &publication_id)
                .await
                .map_err(|cleanup| format!("{error}; voice source cleanup failed: {cleanup}"))?;
        }
        return Err(error);
    }
    crate::spaces::runs::announce_change();
    crate::spaces::runs::load_run(pool, &request.request_id).await
}

async fn create(
    pool: &sqlx::SqlitePool,
    request: &DraftVoiceCandidateRequest,
    frozen: &super::source_snapshot::FrozenConfiguration,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<(), String> {
    if cancellation.is_cancelled() {
        return Err("cancelled".into());
    }
    let mut tx = db::begin_write(pool).await?;
    let raced: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM spaces_runs WHERE id=?)")
        .bind(&request.request_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    if raced {
        return Err("request_conflict: voice candidate request was created concurrently".into());
    }
    let active: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spaces_runs WHERE scope_kind='application' AND completed_at IS NULL \
         AND json_extract(input_snapshot,'$.workflow')=?)",
    )
    .bind(WORKFLOW)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if active {
        return Err("A voice candidate Run is already active".into());
    }
    let ready_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM spaces_voice_candidates WHERE status='ready'")
            .fetch_one(&mut *tx)
            .await
            .map_err(db::sql_error)?;
    if ready_count >= 16 {
        return Err("Reject an existing candidate before creating another".into());
    }
    create_run(
        &mut tx,
        request,
        &frozen.configuration,
        &frozen.request_digest,
    )
    .await?;
    if let Some(publication_id) = &frozen.publication_id {
        super::super::asset_publication::remove_in_transaction(&mut tx, publication_id).await?;
    }
    if cancellation.is_cancelled() {
        return Err("cancelled".into());
    }
    tx.commit().await.map_err(db::sql_error)
}

async fn create_run(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    request: &DraftVoiceCandidateRequest,
    configuration: &VoiceCandidateStepConfiguration,
    request_digest: &str,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let scope = RunScope::Application;
    let grant = local_user_grant(
        uuid::Uuid::new_v4().to_string(),
        scope.clone(),
        Vec::new(),
        now,
    )?;
    let principal = grant.principal.clone();
    create_grant(tx, &grant, now).await?;
    let input = json!({
        "workflow":WORKFLOW,
        "requestDigest":request_digest,
        "referenceSource":configuration.reference_source.clone(),
    });
    create_run_in_transaction(
        tx,
        &NewRun {
            id: &request.request_id,
            scope: &scope,
            parent_run_id: None,
            principal: &principal,
            kind: RunKind::Workflow,
            policy_snapshot: &json!({"grantId":grant.grant_id,"definition":{"failurePolicy":"stop_run"},"workflow":WORKFLOW}),
            input_snapshot: &input,
            budget: &json!({"maxParallelism":1,"maxAttempts":3,"maxStepAttempts":3}),
        },
        now,
    )
    .await?;
    sqlx::query(
        "INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,depends_on,configuration,state,state_version,resolved_input,created_at,updated_at) \
         VALUES(?,?,'candidate','candidate','speech_voice_candidate','[]',?,'ready',0,'{}',?,?)",
    )
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(&request.request_id)
    .bind(sqlx::types::Json(configuration))
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_runs SET expanded_steps=1 WHERE id=?")
        .bind(&request.request_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct VoiceCandidateRunInput {
    workflow: String,
    request_digest: String,
    reference_source: Option<ReferenceSourceSnapshot>,
}

fn request_digest(request: &DraftVoiceCandidateRequest) -> Result<String, String> {
    let bytes = serde_json::to_vec(request).map_err(db::json_error)?;
    Ok(hex::encode(Sha256::digest(bytes)))
}

fn validate_request(request: &DraftVoiceCandidateRequest) -> Result<(), String> {
    if !request.authorized {
        return Err("Voice synthesis requires an explicit authorization declaration".into());
    }
    if !matches!(request.language.as_str(), "zh-CN" | "en")
        || request.text.trim().is_empty()
        || request.text.chars().count() > 500
    {
        return Err("Voice candidate language or sample text is invalid".into());
    }
    if request.model == super::VoiceModel::MossNano
        && !matches!(
            &request.source,
            VoiceCandidateSource::Reference {
                reference_text: None,
                ..
            }
        )
    {
        return Err(
            "MOSS-TTS-Nano supports audio-reference cloning without transcript conditioning".into(),
        );
    }
    match &request.source {
        VoiceCandidateSource::Designed { description }
            if description.trim().is_empty()
                || description.chars().count() > 500
                || description.contains(['(', ')', '\n', '\r']) =>
        {
            Err("Voice design description is invalid".into())
        }
        VoiceCandidateSource::Reference {
            source_path,
            source_revision,
            start_ms,
            end_ms,
            reference_text,
            reference_text_confirmed,
        } => {
            let text = reference_text
                .as_deref()
                .map(str::trim)
                .filter(|text| !text.is_empty());
            if source_revision.len() != 71
                || !source_revision.starts_with("sha256:")
                || end_ms <= start_ms
                || !(5_000..=30_000).contains(&(end_ms - start_ms))
                || *end_ms > 43_200_000
                || source_path.trim().is_empty()
                || text.is_some_and(|text| text.chars().count() > 3_000)
                || (text.is_some() && !reference_text_confirmed)
            {
                return Err(
                    "Reference transcript must be explicitly confirmed before synthesis".into(),
                );
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
#[path = "tests/request.rs"]
mod tests;
