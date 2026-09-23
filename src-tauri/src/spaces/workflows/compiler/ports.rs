use std::collections::{HashMap, HashSet};

use super::{
    diagnostics::{edge_diagnostic, node_diagnostic},
    WorkflowDiagnostic,
};
use crate::spaces::workflows::definition::{WorkflowDefinition, WorkflowNode, WorkflowPort};

pub(super) fn validate(definition: &WorkflowDefinition, diagnostics: &mut Vec<WorkflowDiagnostic>) {
    let nodes = definition
        .nodes
        .iter()
        .map(|node| (node.id(), node))
        .collect::<HashMap<_, _>>();
    let mut branches = HashSet::new();
    let mut edge_ids = HashSet::new();
    let mut incoming = HashMap::<&str, usize>::new();
    for edge in &definition.edges {
        if edge.id.is_empty() || !edge_ids.insert(edge.id.as_str()) {
            diagnostics.push(edge_diagnostic(
                "edge_id",
                "Workflow edge IDs must be nonempty and unique",
                &edge.id,
            ));
        }
        let (Some(source), Some(target)) = (
            nodes.get(edge.source_node_id.as_str()),
            nodes.get(edge.target_node_id.as_str()),
        ) else {
            continue;
        };
        if edge.source_port != WorkflowPort::Output || edge.target_port != WorkflowPort::Input {
            diagnostics.push(edge_diagnostic(
                "port_name",
                "Workflow ports must use registered input/output names",
                &edge.id,
            ));
        }
        validate_branch(
            source,
            edge.branch.as_deref(),
            &edge.id,
            &mut branches,
            diagnostics,
        );
        if !assignable(output_type(source), input_type(target)) {
            diagnostics.push(edge_diagnostic(
                "port_type",
                "Workflow edge connects incompatible port types",
                &edge.id,
            ));
        }
        *incoming.entry(target.id()).or_default() += 1;
    }
    for node in definition
        .nodes
        .iter()
        .filter(|node| matches!(node, WorkflowNode::Condition { .. }))
    {
        for required in ["true", "false"] {
            if !branches.contains(&(node.id().to_owned(), required.to_owned())) {
                diagnostics.push(node_diagnostic(
                    "condition_branch_missing",
                    &format!("Condition requires a {required} branch"),
                    node.id(),
                ));
            }
        }
    }
    for (node_id, count) in incoming {
        let node = nodes[node_id];
        if count > 1
            && !matches!(
                node,
                WorkflowNode::Join { .. } | WorkflowNode::Output { .. }
            )
        {
            diagnostics.push(node_diagnostic(
                "input_cardinality",
                "Only Join and Output accept multiple incoming edges",
                node_id,
            ));
        }
    }
}

fn validate_branch(
    source: &WorkflowNode,
    branch: Option<&str>,
    edge_id: &str,
    branches: &mut HashSet<(String, String)>,
    diagnostics: &mut Vec<WorkflowDiagnostic>,
) {
    if matches!(source, WorkflowNode::Condition { .. }) {
        if !matches!(branch, Some("true" | "false")) {
            diagnostics.push(edge_diagnostic(
                "condition_branch",
                "Condition edges require true or false branch labels",
                edge_id,
            ));
        } else if !branches.insert((
            source.id().to_owned(),
            branch.unwrap_or_default().to_owned(),
        )) {
            diagnostics.push(edge_diagnostic(
                "condition_branch_duplicate",
                "Condition branch labels must be unique",
                edge_id,
            ));
        }
    } else if branch.is_some() {
        diagnostics.push(edge_diagnostic(
            "branch_owner",
            "Only Condition edges may declare a branch",
            edge_id,
        ));
    }
}

fn output_type(node: &WorkflowNode) -> &str {
    match node {
        WorkflowNode::Input { output_type, .. } => output_type.as_str(),
        WorkflowNode::Condition { .. } => "boolean",
        WorkflowNode::ForEach { .. } => "collection",
        WorkflowNode::Action { action_id, .. } => crate::spaces::actions::require(action_id)
            .map(|definition| definition.output_contract.as_str())
            .unwrap_or("value"),
        _ => "value",
    }
}

fn input_type(node: &WorkflowNode) -> &str {
    match node {
        WorkflowNode::Output { input_type, .. } => input_type.as_str(),
        WorkflowNode::ForEach { .. } => "collection",
        _ => "value",
    }
}

fn assignable(source: &str, target: &str) -> bool {
    source == target || source == "value" || target == "value"
}
