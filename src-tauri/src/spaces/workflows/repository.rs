use sha2::{Digest, Sha256};
use sqlx::{Sqlite, SqlitePool, Transaction};

use super::{compile, workflow_dependencies, workflow_rows, WorkflowDraft, WorkflowVersion};
use crate::spaces::db;

pub(super) async fn save_draft(
    pool: &SqlitePool,
    draft: WorkflowDraft,
) -> Result<WorkflowDraft, String> {
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let draft = save_draft_in_transaction(&mut tx, draft, now).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(draft)
}

pub(crate) async fn save_draft_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    mut draft: WorkflowDraft,
    now: i64,
) -> Result<WorkflowDraft, String> {
    validate_identity(&draft)?;
    let definition = serde_json::to_string(&draft.definition).map_err(db::json_error)?;
    let current: Option<i64> =
        sqlx::query_scalar("SELECT revision FROM spaces_workflow_drafts WHERE id=$1")
            .bind(&draft.id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(db::sql_error)?;
    draft.revision = match current {
        None if draft.revision == 0 => {
            sqlx::query("INSERT INTO spaces_workflow_drafts(id,space_id,title,revision,definition,created_at,updated_at) VALUES($1,$2,$3,0,$4,$5,$5)")
                .bind(&draft.id).bind(&draft.space_id).bind(&draft.definition.title)
                .bind(&definition).bind(now).execute(&mut **tx).await.map_err(db::sql_error)?;
            draft.created_at = now;
            0
        }
        Some(revision) if revision == draft.revision => {
            let next = revision + 1;
            let result = sqlx::query("UPDATE spaces_workflow_drafts SET title=$1,definition=$2,revision=$3,updated_at=$4 WHERE id=$5 AND space_id=$6 AND revision=$7")
                .bind(&draft.definition.title).bind(&definition).bind(next).bind(now)
                .bind(&draft.id).bind(&draft.space_id).bind(revision)
                .execute(&mut **tx).await.map_err(db::sql_error)?;
            db::require_one(result, "Workflow draft revision conflict")?;
            next
        }
        None => return Err("Workflow draft must start at revision 0".into()),
        Some(_) => return Err("Workflow draft revision conflict".into()),
    };
    draft.updated_at = now;
    Ok(draft)
}

pub(super) async fn list_drafts(
    pool: &SqlitePool,
    space_id: &str,
) -> Result<Vec<WorkflowDraft>, String> {
    let rows = sqlx::query("SELECT id,space_id,revision,definition,created_at,updated_at FROM spaces_workflow_drafts WHERE space_id=$1 ORDER BY updated_at DESC")
        .bind(space_id).fetch_all(pool).await.map_err(db::sql_error)?;
    rows.iter().map(workflow_rows::parse_draft).collect()
}

pub(super) async fn list_versions(
    pool: &SqlitePool,
    space_id: &str,
) -> Result<Vec<WorkflowVersion>, String> {
    let rows = sqlx::query(
        "SELECT workflow_id,version,space_id,definition_digest,dependency_digest,
        compiler_version,definition,published_at FROM spaces_workflow_versions
        WHERE space_id=$1 ORDER BY published_at DESC",
    )
    .bind(space_id)
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    rows.iter().map(workflow_rows::parse_version).collect()
}

pub(super) async fn delete_draft(pool: &SqlitePool, workflow_id: &str) -> Result<(), String> {
    let result = sqlx::query("DELETE FROM spaces_workflow_drafts WHERE id=$1")
        .bind(workflow_id)
        .execute(pool)
        .await
        .map_err(db::sql_error)?;
    db::require_one(result, "Workflow draft is unavailable")
}

pub(super) async fn publish(
    pool: &SqlitePool,
    workflow_id: &str,
    expected_revision: i64,
) -> Result<WorkflowVersion, String> {
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    let version = publish_in_transaction(&mut tx, workflow_id, expected_revision, now).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(version)
}

pub(crate) async fn publish_in_transaction(
    tx: &mut Transaction<'_, Sqlite>,
    workflow_id: &str,
    expected_revision: i64,
    now: i64,
) -> Result<WorkflowVersion, String> {
    let row = sqlx::query("SELECT id,space_id,revision,definition,created_at,updated_at FROM spaces_workflow_drafts WHERE id=$1")
        .bind(workflow_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?
        .ok_or("Workflow draft is unavailable")?;
    let draft = workflow_rows::parse_draft(&row)?;
    if draft.revision != expected_revision {
        return Err("Workflow draft revision conflict".into());
    }
    let plan = compile(&draft.definition).map_err(|diagnostics| {
        serde_json::to_string(&diagnostics)
            .unwrap_or_else(|_| "Workflow definition is invalid".into())
    })?;
    workflow_dependencies::validate_external_tools(tx, &draft).await?;
    let dependency_digest = workflow_dependencies::closure_digest(tx, &draft).await?;
    let definition_json = serde_json::to_string(&draft.definition).map_err(db::json_error)?;
    let definition_digest = format!("{:x}", Sha256::digest(definition_json.as_bytes()));
    let version: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version),0)+1 FROM spaces_workflow_versions WHERE workflow_id=$1",
    )
    .bind(workflow_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    sqlx::query("INSERT INTO spaces_workflow_versions(workflow_id,version,space_id,definition_digest,dependency_digest,compiler_version,definition,published_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
        .bind(workflow_id).bind(version).bind(&draft.space_id).bind(&definition_digest)
        .bind(&dependency_digest).bind(plan.compiler_version).bind(&definition_json).bind(now)
        .execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(WorkflowVersion {
        workflow_id: workflow_id.to_owned(),
        space_id: draft.space_id,
        version,
        definition_digest,
        dependency_digest,
        compiler_version: plan.compiler_version,
        definition: draft.definition,
        published_at: now,
    })
}

pub(crate) async fn load_version(
    pool: &SqlitePool,
    workflow_id: &str,
    version: i64,
) -> Result<WorkflowVersion, String> {
    let row = sqlx::query(
        "SELECT workflow_id,version,space_id,definition_digest,dependency_digest,
         compiler_version,definition,published_at
         FROM spaces_workflow_versions WHERE workflow_id=$1 AND version=$2",
    )
    .bind(workflow_id)
    .bind(version)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("Workflow version is unavailable")?;
    workflow_rows::parse_version(&row)
}

fn validate_identity(draft: &WorkflowDraft) -> Result<(), String> {
    if draft.id.is_empty() || draft.space_id.is_empty() || draft.definition.title.trim().is_empty()
    {
        return Err("Workflow draft identity and title are required".into());
    }
    if draft.id.encode_utf16().count() > 128 || draft.definition.title.encode_utf16().count() > 512
    {
        return Err("Workflow draft identity or title exceeds its limit".into());
    }
    Ok(())
}
