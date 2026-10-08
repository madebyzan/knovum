use serde_json::json;
use sqlx::{sqlite::SqliteRow, Row, Sqlite, SqlitePool, Transaction};

use super::contracts::GenerateSpeechProjectRequest;
use crate::spaces::{
    authorization::{create_grant, AuthorizationContext, Principal, PrincipalKind},
    db,
    runs::{create_run_in_transaction, NewRun, RunKind, RunObservation},
};

pub(super) const WORKFLOW: &str = "speech_project_generation_v1";

pub(super) async fn start(
    pool: &SqlitePool,
    request: GenerateSpeechProjectRequest,
    runtime_digests: &super::runtime_identity::RuntimeDigests,
) -> Result<RunObservation, String> {
    uuid::Uuid::parse_str(&request.request_id)
        .map_err(|_| "Speech generation request identity must be a UUID")?;
    uuid::Uuid::parse_str(&request.project_id)
        .map_err(|_| "Speech project identity must be a UUID")?;
    super::runtime_identity::validate(runtime_digests)?;
    let mut tx = db::begin_write(pool).await?;
    if let Some(row) = sqlx::query("SELECT input_snapshot FROM spaces_runs WHERE id=?")
        .bind(&request.request_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db::sql_error)?
    {
        let input: String = row.try_get("input_snapshot").map_err(db::sql_error)?;
        let input: serde_json::Value = serde_json::from_str(&input).map_err(db::json_error)?;
        if input.get("workflow").and_then(serde_json::Value::as_str) != Some(WORKFLOW)
            || input.get("projectId").and_then(serde_json::Value::as_str)
                != Some(request.project_id.as_str())
            || input
                .get("expectedRevision")
                .and_then(serde_json::Value::as_i64)
                != Some(request.expected_revision)
        {
            return Err("speech-run-conflict: request identity belongs to another Run".into());
        }
        let frozen_runtime = super::runtime_identity::read_snapshot(&input)?;
        if &frozen_runtime != runtime_digests {
            return Err("runtime_mismatch: Speech Run Python runtime changed".into());
        }
        tx.commit().await.map_err(db::sql_error)?;
        return crate::spaces::runs::load_run(pool, &request.request_id).await;
    }
    let active: Option<(String, String, String, i64, String, String)> = sqlx::query_as(
        "SELECT p.space_id,p.project_type,p.active_revision_id,r.revision,r.output_config,p.source_node_id \
         FROM spaces_speech_projects p JOIN spaces_speech_project_revisions r \
         ON r.id=p.active_revision_id JOIN spaces_nodes n ON n.id=p.source_node_id AND n.space_id=p.space_id \
         WHERE p.id=? AND n.deleted_at IS NULL",
    )
    .bind(&request.project_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let Some((space_id, project_type, revision_id, revision, output_config, source_node_id)) =
        active
    else {
        return Err("Speech project no longer exists".into());
    };
    if revision != request.expected_revision {
        return Err("stale_revision: speech project changed before generation".into());
    }
    let active_run: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM spaces_runs WHERE completed_at IS NULL \
         AND json_extract(input_snapshot,'$.workflow')=? \
         AND json_extract(input_snapshot,'$.projectId')=?)",
    )
    .bind(WORKFLOW)
    .bind(&request.project_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if active_run {
        return Err("Speech project already has an active Run".into());
    }
    let output_config: serde_json::Value =
        serde_json::from_str(&output_config).map_err(db::json_error)?;
    let config = super::project_config::parse(&project_type, &output_config)?;
    super::project_source::require(&mut tx, &space_id, &source_node_id, &config).await?;
    let segments = sqlx::query(
        "SELECT s.id,s.ordinal,s.chapter_key,s.speaker_key,s.voice_revision_id,v.engine_id \
         FROM spaces_speech_segments s JOIN spaces_voice_revisions v ON v.id=s.voice_revision_id \
         WHERE s.project_revision_id=? ORDER BY s.ordinal",
    )
    .bind(&revision_id)
    .fetch_all(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if segments.is_empty() {
        return Err("Speech project has no segments to generate".into());
    }
    let profiles = segments
        .iter()
        .map(|row| {
            let engine: String = row.try_get("engine_id").map_err(db::sql_error)?;
            super::runtime_identity::profile_for_engine(&engine)
        })
        .collect::<Result<std::collections::BTreeSet<_>, String>>()?;
    if profiles != runtime_digests.keys().copied().collect() {
        return Err("runtime_mismatch: speech project runtime packages changed".into());
    }
    if let super::project_config::ProjectOutputConfig::Audiobook(config) = &config {
        validate_audiobook_segments(config, &segments)?;
    }
    create(
        &mut tx,
        &request,
        &space_id,
        &project_type,
        &revision_id,
        &segments,
        runtime_digests,
    )
    .await?;
    tx.commit().await.map_err(db::sql_error)?;
    crate::spaces::runs::announce_change();
    crate::spaces::runs::load_run(pool, &request.request_id).await
}

fn validate_audiobook_segments(
    config: &super::project_config::AudiobookOutputConfig,
    segments: &[SqliteRow],
) -> Result<(), String> {
    let keys = segments
        .iter()
        .map(|row| row.try_get::<Option<String>, _>("chapter_key"))
        .collect::<Result<Vec<_>, _>>()
        .map_err(db::sql_error)?;
    super::project_config::validate_chapter_order(config, keys.iter().map(Option::as_deref))?;
    for row in segments {
        let speaker_key = row
            .try_get::<Option<String>, _>("speaker_key")
            .map_err(db::sql_error)?;
        let voice_revision_id = row
            .try_get::<String, _>("voice_revision_id")
            .map_err(db::sql_error)?;
        if super::project_config::role_voice(config, speaker_key.as_deref())?
            != voice_revision_id.as_str()
        {
            return Err("Audiobook paragraph voice differs from its role binding".into());
        }
    }
    Ok(())
}

async fn create(
    tx: &mut Transaction<'_, Sqlite>,
    request: &GenerateSpeechProjectRequest,
    space_id: &str,
    project_type: &str,
    revision_id: &str,
    segments: &[sqlx::sqlite::SqliteRow],
    runtime_digests: &super::runtime_identity::RuntimeDigests,
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
    let input = json!({
        "workflow": WORKFLOW,
        "projectId": request.project_id,
        "revisionId": revision_id,
        "expectedRevision": request.expected_revision,
        "projectType": project_type,
        "runtimeDigests": runtime_digests,
    });
    let policy = json!({
        "grantId": grant.grant_id,
        "definition": {"failurePolicy": "stop_run"},
        "workflow": WORKFLOW,
    });
    let chapter_count = if project_type == "audiobook" {
        let mut count = 0_usize;
        let mut previous = None;
        for row in segments {
            let key: Option<String> = row.try_get("chapter_key").map_err(db::sql_error)?;
            if key.as_deref() != previous.as_deref() {
                count += 1;
            }
            previous = key;
        }
        count
    } else {
        0
    };
    let step_count = segments
        .len()
        .checked_add(if project_type == "audiobook" {
            chapter_count
                .checked_add(2)
                .ok_or("Speech Run step count overflow")?
        } else {
            1
        })
        .ok_or("Speech Run step count overflow")?;
    let attempts = i64::try_from(step_count)
        .map_err(|_| "Speech project contains too many segments")?
        .checked_mul(3)
        .ok_or("Speech Run attempt budget overflow")?;
    let budget = json!({"maxParallelism": 1, "maxAttempts": 3, "maxStepAttempts": attempts});
    create_run_in_transaction(
        tx,
        &NewRun {
            id: &request.request_id,
            home_space_id: space_id,
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
    let mut dependencies = Vec::with_capacity(segments.len());
    for row in segments {
        let segment_id: String = row.try_get("id").map_err(db::sql_error)?;
        let ordinal: i64 = row.try_get("ordinal").map_err(db::sql_error)?;
        let node_id = format!("segment:{segment_id}");
        insert_step(tx, &request.request_id, &node_id, "speech_segment", &[],
            &json!({"projectId":request.project_id,"revisionId":revision_id,"segmentId":segment_id,"ordinal":ordinal}), now).await?;
        dependencies.push(node_id);
    }
    if project_type == "audiobook" {
        let mut chapters: Vec<(String, Vec<String>)> = Vec::new();
        for row in segments {
            let segment_id: String = row.try_get("id").map_err(db::sql_error)?;
            let chapter_key: String = row
                .try_get::<Option<String>, _>("chapter_key")
                .map_err(db::sql_error)?
                .ok_or("Audiobook segment has no chapter key")?;
            if chapters
                .last()
                .is_none_or(|chapter| chapter.0 != chapter_key)
            {
                chapters.push((chapter_key.clone(), Vec::new()));
            }
            chapters
                .last_mut()
                .ok_or("Audiobook chapter grouping failed")?
                .1
                .push(format!("segment:{segment_id}"));
        }
        let mut chapter_nodes = Vec::with_capacity(chapters.len());
        for (index, (chapter_key, segment_nodes)) in chapters.iter().enumerate() {
            let node_id = format!("chapter:{index}");
            insert_step(tx, &request.request_id, &node_id, "speech_chapter_pcm", segment_nodes,
                &json!({"projectId":request.project_id,"revisionId":revision_id,"revision":request.expected_revision,"chapterKey":chapter_key,"chapterIndex":index}), now).await?;
            chapter_nodes.push(node_id);
        }
        insert_step(tx, &request.request_id, "loudness", "speech_loudness_measurement", &chapter_nodes,
            &json!({"projectId":request.project_id,"revisionId":revision_id,"revision":request.expected_revision}), now).await?;
        insert_step(tx, &request.request_id, "assembly", "speech_audiobook_encode", &["loudness".into()],
            &json!({"projectId":request.project_id,"revisionId":revision_id,"revision":request.expected_revision,"projectType":project_type}), now).await?;
    } else {
        insert_step(tx, &request.request_id, "assembly", "speech_assembly", &dependencies,
            &json!({"projectId":request.project_id,"revisionId":revision_id,"revision":request.expected_revision,"projectType":project_type}), now).await?;
    }
    sqlx::query("UPDATE spaces_runs SET expanded_steps=? WHERE id=?")
        .bind(i64::try_from(step_count).map_err(|_| "Speech Run step count overflow")?)
        .bind(&request.request_id)
        .execute(&mut **tx)
        .await
        .map_err(db::sql_error)?;
    Ok(())
}

async fn insert_step(
    tx: &mut Transaction<'_, Sqlite>,
    run_id: &str,
    node_id: &str,
    node_kind: &str,
    dependencies: &[String],
    configuration: &serde_json::Value,
    now: i64,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO spaces_run_steps(id,run_id,definition_node_id,instance_path,node_kind,depends_on,configuration,state,state_version,resolved_input,created_at,updated_at) \
         VALUES(?,?,?,?,?,?,?, ?,0,'{}',?,?)",
    )
    .bind(uuid::Uuid::new_v4().to_string()).bind(run_id).bind(node_id).bind(node_id)
    .bind(node_kind).bind(serde_json::to_string(dependencies).map_err(db::json_error)?)
    .bind(serde_json::to_string(configuration).map_err(db::json_error)?)
    .bind(if dependencies.is_empty() { "ready" } else { "pending" }).bind(now).bind(now)
    .execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}
