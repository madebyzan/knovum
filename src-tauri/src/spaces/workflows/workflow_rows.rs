use sqlx::Row;

use super::{WorkflowDefinition, WorkflowDraft, WorkflowVersion};
use crate::spaces::db;

pub(super) fn parse_version(row: &sqlx::sqlite::SqliteRow) -> Result<WorkflowVersion, String> {
    let definition: String = row.try_get("definition").map_err(db::sql_error)?;
    Ok(WorkflowVersion {
        workflow_id: row.try_get("workflow_id").map_err(db::sql_error)?,
        version: row.try_get("version").map_err(db::sql_error)?,
        space_id: row.try_get("space_id").map_err(db::sql_error)?,
        definition_digest: row.try_get("definition_digest").map_err(db::sql_error)?,
        dependency_digest: row.try_get("dependency_digest").map_err(db::sql_error)?,
        compiler_version: row.try_get("compiler_version").map_err(db::sql_error)?,
        definition: serde_json::from_str(&definition).map_err(db::json_error)?,
        published_at: row.try_get("published_at").map_err(db::sql_error)?,
    })
}

pub(super) fn parse_draft(row: &sqlx::sqlite::SqliteRow) -> Result<WorkflowDraft, String> {
    let definition: String = row.try_get("definition").map_err(db::sql_error)?;
    Ok(WorkflowDraft {
        id: row.try_get("id").map_err(db::sql_error)?,
        space_id: row.try_get("space_id").map_err(db::sql_error)?,
        revision: row.try_get("revision").map_err(db::sql_error)?,
        definition: serde_json::from_str::<WorkflowDefinition>(&definition)
            .map_err(db::json_error)?,
        created_at: row.try_get("created_at").map_err(db::sql_error)?,
        updated_at: row.try_get("updated_at").map_err(db::sql_error)?,
    })
}
