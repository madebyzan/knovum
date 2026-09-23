use serde_json::Value;

use crate::spaces::workflows::{
    definition::{
        FailurePolicy, JoinPolicy, WorkflowEdge, WorkflowLimits, WorkflowNode, WorkflowPort,
        WorkflowValueType,
    },
    WorkflowDefinition,
};

pub(super) fn base(
    title: &str,
    nodes: Vec<WorkflowNode>,
    edges: Vec<WorkflowEdge>,
) -> WorkflowDefinition {
    WorkflowDefinition {
        schema_version: 1,
        title: title.into(),
        nodes,
        edges,
        layout: Default::default(),
        failure_policy: FailurePolicy::StopRun,
        limits: WorkflowLimits {
            max_expanded_steps: 100,
            max_parallelism: 4,
            max_duration_seconds: 86_400,
            max_total_tokens: 250_000,
            max_step_attempts: 500,
            max_input_bytes: 67_108_864,
            max_output_bytes: 268_435_456,
        },
    }
}

pub(super) fn input() -> WorkflowNode {
    WorkflowNode::Input {
        id: "input".into(),
        output_type: WorkflowValueType::Value,
    }
}

pub(super) fn output() -> WorkflowNode {
    WorkflowNode::Output {
        id: "output".into(),
        input_type: WorkflowValueType::Value,
    }
}

pub(super) fn join(id: &str) -> WorkflowNode {
    WorkflowNode::Join {
        id: id.into(),
        policy: JoinPolicy::AllSucceeded,
    }
}

pub(super) fn review(id: &str, purpose: &str) -> WorkflowNode {
    WorkflowNode::Review {
        id: id.into(),
        purpose: purpose.into(),
    }
}

pub(super) fn action(id: &str, action_id: &str, parameters: Value) -> Result<WorkflowNode, String> {
    let definition = crate::spaces::actions::require(action_id)?;
    crate::spaces::actions::validate_parameters(definition, &parameters)?;
    Ok(WorkflowNode::Action {
        id: id.into(),
        action_id: action_id.into(),
        action_version: 1,
        parameters,
    })
}

pub(super) fn edges(links: &[(&str, &str)]) -> Vec<WorkflowEdge> {
    links
        .iter()
        .enumerate()
        .map(|(index, (source, target))| WorkflowEdge {
            id: format!("edge-{index}"),
            source_node_id: (*source).into(),
            source_port: WorkflowPort::Output,
            target_node_id: (*target).into(),
            target_port: WorkflowPort::Input,
            branch: None,
        })
        .collect()
}
