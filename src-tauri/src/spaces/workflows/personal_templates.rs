use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use ts_rs::TS;

use super::{WorkflowDefinition, WorkflowDraft};
use crate::spaces::db;

#[derive(Clone, Debug, Deserialize, TS)]
#[ts(export_to = "SaveWorkflowTemplateRequest.ts")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SaveWorkflowTemplateRequest {
    pub id: String,
    pub name: String,
    pub definition: WorkflowDefinition,
    pub source_workflow_id: Option<String>,
    pub source_workflow_version: Option<i64>,
}

#[derive(Clone, Debug, Serialize, TS)]
#[ts(export_to = "WorkflowTemplate.ts")]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkflowTemplate {
    pub id: String,
    pub name: String,
    pub definition: WorkflowDefinition,
    pub dependencies: Vec<super::template_dependencies::TemplateDependency>,
    pub source_workflow_id: Option<String>,
    pub source_workflow_version: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

pub(crate) async fn save(
    pool: &SqlitePool,
    request: SaveWorkflowTemplateRequest,
) -> Result<WorkflowTemplate, String> {
    if request.id.trim().is_empty() || request.name.trim().is_empty() {
        return Err("workflow-template-invalid: identity and name are required".into());
    }
    super::compile(&request.definition).map_err(|diagnostics| {
        serde_json::to_string(&diagnostics).unwrap_or_else(|_| "Workflow is invalid".into())
    })?;
    let dependencies = super::template_dependencies::collect(pool, &request).await?;
    let now = crate::clock::unix_time_millis()?;
    sqlx::query(
        "INSERT INTO spaces_workflow_templates(id,name,definition,dependencies,source_workflow_id,
         source_workflow_version,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,$7,$7)
         ON CONFLICT(id) DO UPDATE SET name=excluded.name,definition=excluded.definition,
         dependencies=excluded.dependencies,
         source_workflow_id=excluded.source_workflow_id,
         source_workflow_version=excluded.source_workflow_version,updated_at=excluded.updated_at",
    )
    .bind(&request.id)
    .bind(request.name.trim())
    .bind(serde_json::to_string(&request.definition).map_err(db::json_error)?)
    .bind(serde_json::to_string(&dependencies).map_err(db::json_error)?)
    .bind(request.source_workflow_id)
    .bind(request.source_workflow_version)
    .bind(now)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    load(pool, &request.id).await
}

pub(crate) async fn list(pool: &SqlitePool) -> Result<Vec<WorkflowTemplate>, String> {
    let rows = sqlx::query("SELECT * FROM spaces_workflow_templates ORDER BY updated_at DESC")
        .fetch_all(pool)
        .await
        .map_err(db::sql_error)?;
    rows.iter().map(parse).collect()
}

pub(crate) async fn delete(pool: &SqlitePool, id: &str) -> Result<(), String> {
    let result = sqlx::query("DELETE FROM spaces_workflow_templates WHERE id=$1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(db::sql_error)?;
    db::require_one(
        result,
        "workflow-template-unavailable: template is unavailable",
    )
}

pub(crate) async fn copy(
    pool: &SqlitePool,
    template_id: &str,
    space_id: &str,
) -> Result<WorkflowDraft, String> {
    let template = load(pool, template_id).await?;
    super::template_dependencies::instantiate(pool, template, space_id).await
}

async fn load(pool: &SqlitePool, id: &str) -> Result<WorkflowTemplate, String> {
    let row = sqlx::query("SELECT * FROM spaces_workflow_templates WHERE id=$1")
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(db::sql_error)?
        .ok_or("workflow-template-unavailable: template is unavailable")?;
    parse(&row)
}

fn parse(row: &sqlx::sqlite::SqliteRow) -> Result<WorkflowTemplate, String> {
    let definition: String = row.try_get("definition").map_err(db::sql_error)?;
    let dependencies: String = row.try_get("dependencies").map_err(db::sql_error)?;
    Ok(WorkflowTemplate {
        id: row.try_get("id").map_err(db::sql_error)?,
        name: row.try_get("name").map_err(db::sql_error)?,
        definition: serde_json::from_str(&definition).map_err(db::json_error)?,
        dependencies: serde_json::from_str(&dependencies).map_err(db::json_error)?,
        source_workflow_id: row.try_get("source_workflow_id").map_err(db::sql_error)?,
        source_workflow_version: row
            .try_get("source_workflow_version")
            .map_err(db::sql_error)?,
        created_at: row.try_get("created_at").map_err(db::sql_error)?,
        updated_at: row.try_get("updated_at").map_err(db::sql_error)?,
    })
}
