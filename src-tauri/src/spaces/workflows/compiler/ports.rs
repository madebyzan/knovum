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
        if edge.branch.is_some() {
            diagnostics.push(edge_diagnostic(
                "branch_unsupported",
                "Workflow edges cannot declare branches",
                &edge.id,
            ));
        }
        if !assignable(output_type(source), input_type(target)) {
            diagnostics.push(edge_diagnostic(
                "port_type",
                "Workflow edge connects incompatible port types",
                &edge.id,
            ));
        }
        *incoming.entry(target.id()).or_default() += 1;
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

fn output_type(node: &WorkflowNode) -> &str {
    match node {
        WorkflowNode::Input { output_type, .. } => output_type.as_str(),
        WorkflowNode::Action { action_id, .. } => crate::spaces::actions::require(action_id)
            .map(|definition| definition.output_contract.as_str())
            .unwrap_or("value"),
        _ => "value",
    }
}

fn input_type(node: &WorkflowNode) -> &str {
    match node {
        WorkflowNode::Output { input_type, .. } => input_type.as_str(),
        _ => "value",
    }
}

fn assignable(source: &str, target: &str) -> bool {
    source == target || source == "value" || target == "value"
}
