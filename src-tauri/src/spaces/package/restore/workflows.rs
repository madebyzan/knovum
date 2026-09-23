use sha2::{Digest, Sha256};
use sqlx::{Sqlite, Transaction};

use super::contracts::RestorePackageRequest;
use crate::spaces::db;

pub(super) async fn insert(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.workflow_drafts {
        crate::spaces::workflows::compile(&row.definition)
            .map_err(|_| "Package Workflow draft is invalid")?;
        let definition = serde_json::to_string(&row.definition).map_err(db::json_error)?;
        sqlx::query("INSERT INTO spaces_workflow_drafts(id,space_id,title,revision,definition,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,$7)")
            .bind(&row.id).bind(&request.space.id).bind(&row.definition.title).bind(row.revision)
            .bind(definition).bind(row.created_at).bind(row.updated_at)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    for row in &request.workflow_versions {
        let plan = crate::spaces::workflows::compile(&row.definition)
            .map_err(|_| "Package Workflow version is invalid")?;
        let definition = serde_json::to_string(&row.definition).map_err(db::json_error)?;
        let definition_digest = format!("{:x}", Sha256::digest(definition.as_bytes()));
        sqlx::query("INSERT INTO spaces_workflow_versions(workflow_id,version,space_id,definition_digest,dependency_digest,compiler_version,definition,published_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
            .bind(&row.workflow_id).bind(row.version).bind(&request.space.id)
            .bind(definition_digest).bind(plan.dependency_digest).bind(plan.compiler_version)
            .bind(definition).bind(row.published_at).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}
