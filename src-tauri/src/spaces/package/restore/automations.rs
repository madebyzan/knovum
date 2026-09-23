use sqlx::{Sqlite, Transaction};

use super::contracts::RestorePackageRequest;
use crate::spaces::{
    authorization::{create_grant, AuthorizationContext, Principal, PrincipalKind},
    db,
};

pub(super) async fn insert(
    tx: &mut Transaction<'_, Sqlite>,
    request: &RestorePackageRequest,
) -> Result<(), String> {
    for row in &request.automation_triggers {
        let definition = request
            .workflow_versions
            .iter()
            .find(|version| {
                version.workflow_id == row.workflow_id && version.version == row.workflow_version
            })
            .ok_or("Package Automation references an unavailable Workflow version")?;
        let grant = grant(request, row, &definition.definition.nodes);
        create_grant(tx, &grant, row.created_at).await?;
        sqlx::query("INSERT INTO spaces_automation_triggers(id,space_id,name,workflow_id,workflow_version,grant_id,source_kind,source_config,input,overlap_policy,enabled,valid_until,next_scheduled_at,created_at,updated_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,0,0,NULL,$11,$12)")
            .bind(&row.id).bind(&request.space.id).bind(&row.name).bind(&row.workflow_id)
            .bind(row.workflow_version).bind(&grant.grant_id).bind(source_kind(&row.source)?)
            .bind(json(&row.source)?).bind(json(&row.input)?).bind(&row.overlap_policy)
            .bind(row.created_at).bind(row.updated_at).execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    for row in &request.automation_occurrences {
        sqlx::query("INSERT INTO spaces_automation_occurrences(id,trigger_id,scheduled_at,occurrence_key,run_id,state,detail,created_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8)")
            .bind(&row.id).bind(&row.trigger_id).bind(row.scheduled_at).bind(&row.occurrence_key)
            .bind(&row.run_id).bind(&row.state).bind(json(&row.detail)?).bind(row.created_at)
            .execute(&mut **tx).await.map_err(db::sql_error)?;
    }
    Ok(())
}

fn grant(
    request: &RestorePackageRequest,
    row: &super::contracts::PackageAutomationTriggerRow,
    nodes: &[crate::spaces::workflows::definition::WorkflowNode],
) -> AuthorizationContext {
    let action_ids = nodes
        .iter()
        .filter_map(|node| match node {
            crate::spaces::workflows::definition::WorkflowNode::Action { action_id, .. } => {
                Some(action_id.clone())
            }
            _ => None,
        })
        .collect();
    let egress_targets = nodes
        .iter()
        .filter_map(|node| match node {
            crate::spaces::workflows::definition::WorkflowNode::ExternalTool {
                binding_id, ..
            } => Some(format!("external-tool:{binding_id}")),
            _ => None,
        })
        .collect();
    AuthorizationContext {
        principal: Principal {
            kind: PrincipalKind::Automation,
            id: row.id.clone(),
        },
        grant_id: uuid::Uuid::new_v4().to_string(),
        home_space_id: request.space.id.clone(),
        read_space_ids: vec![request.space.id.clone()],
        write_space_ids: vec![request.space.id.clone()],
        action_ids,
        egress_targets,
        valid_until: 0,
    }
}

fn source_kind(value: &serde_json::Value) -> Result<&str, String> {
    match value.get("kind").and_then(serde_json::Value::as_str) {
        Some(kind @ ("cron" | "event")) => Ok(kind),
        _ => Err("Package Automation source is invalid".into()),
    }
}

fn json(value: &serde_json::Value) -> Result<String, String> {
    serde_json::to_string(value).map_err(db::json_error)
}
