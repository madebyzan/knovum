use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use ts_rs::TS;

use super::definition::WorkflowNode;
use super::personal_templates::{SaveWorkflowTemplateRequest, WorkflowTemplate};
use super::{WorkflowDefinition, WorkflowDraft};
use crate::spaces::db;

mod graph;

use graph::{order_dependencies, references, reject_external_tools, rewrite_references};

#[derive(Clone, Debug, Deserialize, Serialize, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct TemplateDependency {
    pub workflow_id: String,
    pub version: u32,
    pub definition: WorkflowDefinition,
}

pub(super) async fn collect(
    pool: &SqlitePool,
    request: &SaveWorkflowTemplateRequest,
) -> Result<Vec<TemplateDependency>, String> {
    reject_external_tools(&request.definition)?;
    let mut queue = references(&request.definition);
    let mut dependencies = BTreeMap::new();
    let mut source_space = None;
    while let Some(key) = queue.pop() {
        if dependencies.contains_key(&key) {
            continue;
        }
        if request.source_workflow_id.as_deref() == Some(&key.0) {
            return Err(
                "workflow-template-recursion-forbidden: dependency returns to the template root"
                    .into(),
            );
        }
        let row = sqlx::query(
            "SELECT space_id,definition FROM spaces_workflow_versions WHERE workflow_id=$1 AND version=$2",
        ).bind(&key.0).bind(i64::from(key.1)).fetch_optional(pool).await.map_err(db::sql_error)?
            .ok_or("workflow-template-dependency-unavailable: referenced version is unavailable")?;
        let space_id: String = row.try_get("space_id").map_err(db::sql_error)?;
        if source_space.get_or_insert_with(|| space_id.clone()) != &space_id {
            return Err(
                "workflow-template-cross-space-dependency: dependencies must share one Space"
                    .into(),
            );
        }
        let definition = serde_json::from_str::<WorkflowDefinition>(
            &row.try_get::<String, _>("definition")
                .map_err(db::sql_error)?,
        )
        .map_err(db::json_error)?;
        reject_external_tools(&definition)?;
        queue.extend(references(&definition));
        dependencies.insert(
            key.clone(),
            TemplateDependency {
                workflow_id: key.0,
                version: key.1,
                definition,
            },
        );
        if dependencies.len() > 64 {
            return Err(
                "workflow-template-closure-limit: dependency closure exceeds 64 versions".into(),
            );
        }
    }
    Ok(dependencies.into_values().collect())
}

pub(super) async fn instantiate(
    pool: &SqlitePool,
    template: WorkflowTemplate,
    space_id: &str,
) -> Result<WorkflowDraft, String> {
    let mappings = template
        .dependencies
        .iter()
        .map(|dependency| {
            (
                (dependency.workflow_id.clone(), dependency.version),
                uuid::Uuid::new_v4().to_string(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let ordered = order_dependencies(&template.dependencies)?;
    let now = crate::clock::unix_time_millis()?;
    let mut tx = db::begin_write(pool).await?;
    for dependency in ordered {
        let id = mappings
            .get(&(dependency.workflow_id.clone(), dependency.version))
            .ok_or("workflow-template-dependency-map-missing")?;
        let definition = rewrite_references(dependency.definition.clone(), &mappings)?;
        insert_dependency_draft(&mut tx, id, space_id, &definition, now).await?;
        super::repository::publish_in_transaction(&mut tx, id, 0, now).await?;
        sqlx::query("DELETE FROM spaces_workflow_drafts WHERE id=$1")
            .bind(id)
            .execute(&mut *tx)
            .await
            .map_err(db::sql_error)?;
    }
    let definition = rewrite_references(template.definition, &mappings)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(WorkflowDraft {
        id: uuid::Uuid::new_v4().to_string(),
        space_id: space_id.to_owned(),
        revision: 0,
        definition,
        created_at: 0,
        updated_at: 0,
    })
}

async fn insert_dependency_draft(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    id: &str,
    space_id: &str,
    definition: &WorkflowDefinition,
    now: i64,
) -> Result<(), String> {
    sqlx::query(
        "INSERT INTO spaces_workflow_drafts(id,space_id,title,revision,definition,created_at,updated_at)
         VALUES($1,$2,$3,0,$4,$5,$5)",
    ).bind(id).bind(space_id).bind(&definition.title)
        .bind(serde_json::to_string(definition).map_err(db::json_error)?)
        .bind(now).execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}

#[cfg(test)]
mod tests;
