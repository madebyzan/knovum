use sha2::{Digest, Sha256};
use sqlx::{Row, Sqlite, Transaction};

use super::{WorkflowDefinition, WorkflowDraft};
use crate::spaces::db;

pub(super) async fn validate_external_tools(
    tx: &mut Transaction<'_, Sqlite>,
    draft: &WorkflowDraft,
) -> Result<(), String> {
    for node in &draft.definition.nodes {
        let super::definition::WorkflowNode::ExternalTool {
            binding_id,
            schema_fingerprint,
            ..
        } = node
        else {
            continue;
        };
        let row = sqlx::query(
            "SELECT space_id,schema_fingerprint,enabled FROM spaces_external_tool_bindings WHERE id=$1",
        ).bind(binding_id).fetch_optional(&mut **tx).await.map_err(db::sql_error)?
            .ok_or("external-tool-unavailable: Workflow binding is unavailable")?;
        let space_id: String = row.try_get("space_id").map_err(db::sql_error)?;
        let fingerprint: String = row.try_get("schema_fingerprint").map_err(db::sql_error)?;
        let enabled: bool = row.try_get("enabled").map_err(db::sql_error)?;
        if space_id != draft.space_id || !enabled {
            return Err(
                "external-tool-unavailable: binding is disabled or belongs to another Space".into(),
            );
        }
        if fingerprint != *schema_fingerprint {
            return Err(
                "external-tool-schema-changed: Workflow must review the current tool schema".into(),
            );
        }
    }
    Ok(())
}

pub(super) async fn closure_digest(
    tx: &mut Transaction<'_, Sqlite>,
    draft: &WorkflowDraft,
) -> Result<String, String> {
    let mut queue = draft
        .definition
        .nodes
        .iter()
        .filter_map(subworkflow_ref)
        .collect::<Vec<_>>();
    let mut visited = std::collections::BTreeMap::<(String, i64), String>::new();
    while let Some((workflow_id, version)) = queue.pop() {
        if workflow_id == draft.id {
            return Err(
                "workflow-recursion-forbidden: dependency returns to the publishing Workflow"
                    .into(),
            );
        }
        if visited.contains_key(&(workflow_id.clone(), version)) {
            continue;
        }
        let row = sqlx::query(
            "SELECT space_id,definition_digest,definition FROM spaces_workflow_versions
             WHERE workflow_id=$1 AND version=$2",
        )
        .bind(&workflow_id)
        .bind(version)
        .fetch_optional(&mut **tx)
        .await
        .map_err(db::sql_error)?
        .ok_or("subworkflow-version-unavailable: referenced version is unavailable")?;
        let space_id: String = row.try_get("space_id").map_err(db::sql_error)?;
        if space_id != draft.space_id {
            return Err("permission-denied: subworkflow belongs to another Space".into());
        }
        let definition: WorkflowDefinition = serde_json::from_str(
            &row.try_get::<String, _>("definition")
                .map_err(db::sql_error)?,
        )
        .map_err(db::json_error)?;
        queue.extend(definition.nodes.iter().filter_map(subworkflow_ref));
        visited.insert(
            (workflow_id, version),
            row.try_get("definition_digest").map_err(db::sql_error)?,
        );
        if visited.len() > 64 {
            return Err("subworkflow-closure-limit: dependency closure exceeds 64 versions".into());
        }
    }
    let encoded = visited
        .into_iter()
        .map(|((id, version), digest)| format!("{id}:{version}:{digest}"))
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!("{:x}", Sha256::digest(encoded)))
}

fn subworkflow_ref(node: &super::definition::WorkflowNode) -> Option<(String, i64)> {
    match node {
        super::definition::WorkflowNode::Subworkflow {
            workflow_id,
            version,
            ..
        } => Some((workflow_id.clone(), i64::from(*version))),
        _ => None,
    }
}
