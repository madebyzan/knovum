use super::{AutomationSource, SaveAutomationTriggerRequest};
use crate::spaces::{
    authorization::{AuthorizationContext, Principal, PrincipalKind},
    db,
    workflows::definition::WorkflowNode,
};

pub(super) fn trigger_grant(
    request: &SaveAutomationTriggerRequest,
    nodes: &[WorkflowNode],
    now: i64,
) -> AuthorizationContext {
    let action_ids = nodes
        .iter()
        .filter_map(|node| match node {
            WorkflowNode::Action { action_id, .. } => Some(action_id.clone()),
            _ => None,
        })
        .collect();
    let egress_targets = nodes
        .iter()
        .filter_map(|node| match node {
            WorkflowNode::ExternalTool { binding_id, .. } => {
                Some(format!("external-tool:{binding_id}"))
            }
            _ => None,
        })
        .collect();
    AuthorizationContext {
        principal: Principal {
            kind: PrincipalKind::Automation,
            id: request.id.clone(),
        },
        grant_id: uuid::Uuid::new_v4().to_string(),
        home_space_id: request.space_id.clone(),
        read_space_ids: vec![request.space_id.clone()],
        write_space_ids: vec![request.space_id.clone()],
        action_ids,
        egress_targets,
        valid_until: request.valid_until.max(now),
    }
}

pub(super) fn validate(request: &SaveAutomationTriggerRequest) -> Result<(), String> {
    if request.id.trim().is_empty()
        || request.space_id.trim().is_empty()
        || request.name.trim().is_empty()
    {
        return Err("automation-trigger-invalid: identity and name are required".into());
    }
    if request.workflow_version == 0
        || request.valid_until <= crate::clock::unix_time_millis()?
        || !valid_source(&request.source)
    {
        return Err("automation-trigger-invalid: version, catch-up limit, or authorization expiry is invalid".into());
    }
    Ok(())
}

fn valid_source(source: &AutomationSource) -> bool {
    match source {
        AutomationSource::Cron {
            misfire_window_seconds,
            catch_up_limit,
            ..
        } => *catch_up_limit <= 100 && *misfire_window_seconds <= 604_800,
        AutomationSource::Event {
            event_kind,
            causal_depth_limit,
            rate_limit_count,
            rate_limit_window_seconds,
            ..
        } => {
            !event_kind.as_str().is_empty()
                && (1..=16).contains(causal_depth_limit)
                && (1..=1_000).contains(rate_limit_count)
                && (1..=86_400).contains(rate_limit_window_seconds)
        }
    }
}

pub(super) fn source_kind(source: &AutomationSource) -> &'static str {
    match source {
        AutomationSource::Cron { .. } => "cron",
        AutomationSource::Event { .. } => "event",
    }
}

pub(super) fn enum_text(value: impl serde::Serialize) -> Result<String, String> {
    serde_json::to_value(value)
        .map_err(db::json_error)?
        .as_str()
        .map(str::to_owned)
        .ok_or("Invalid enum".into())
}
