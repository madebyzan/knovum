use std::collections::{HashMap, HashSet};

use petgraph::{graph::DiGraph, visit::Walker};

use super::{
    diagnostics::{global_diagnostic, node_diagnostic},
    WorkflowDiagnostic,
};
use crate::spaces::workflows::definition::{WorkflowDefinition, WorkflowNode};

mod limits;

const MAX_DEFINITION_NODES: usize = 200;

pub(super) fn definition(definition: &WorkflowDefinition) -> Vec<WorkflowDiagnostic> {
    let mut diagnostics = Vec::new();
    if definition.schema_version != 1 {
        diagnostics.push(global_diagnostic(
            "schema_version",
            "Workflow schemaVersion must be 1",
        ));
    }
    if definition.nodes.is_empty() || definition.nodes.len() > MAX_DEFINITION_NODES {
        diagnostics.push(global_diagnostic(
            "node_count",
            "Workflow requires 1 to 200 definition nodes",
        ));
    }
    let mut ids = HashSet::new();
    for node in &definition.nodes {
        if node.id().is_empty() || !ids.insert(node.id()) {
            diagnostics.push(node_diagnostic(
                "node_id",
                "Workflow node IDs must be nonempty and unique",
                node.id(),
            ));
        }
        validate_action(node, &mut diagnostics);
    }
    if !definition
        .nodes
        .iter()
        .any(|node| matches!(node, WorkflowNode::Input { .. }))
    {
        diagnostics.push(global_diagnostic(
            "input_missing",
            "Workflow requires an Input node",
        ));
    }
    if !definition
        .nodes
        .iter()
        .any(|node| matches!(node, WorkflowNode::Output { .. }))
    {
        diagnostics.push(global_diagnostic(
            "output_missing",
            "Workflow requires an Output node",
        ));
    }
    limits::validate(definition, &mut diagnostics);
    diagnostics
}

fn validate_action(node: &WorkflowNode, diagnostics: &mut Vec<WorkflowDiagnostic>) {
    let WorkflowNode::Action {
        action_id,
        action_version,
        parameters,
        ..
    } = node
    else {
        return;
    };
    match crate::spaces::actions::require(action_id) {
        Ok(definition) if definition.version == *action_version => {
            if let Err(message) =
                crate::spaces::actions::validate_parameters(definition, parameters)
            {
                diagnostics.push(node_diagnostic("action_parameters", &message, node.id()));
            }
        }
        Ok(_) => diagnostics.push(node_diagnostic(
            "action_version",
            "SpaceAction version is unavailable",
            node.id(),
        )),
        Err(message) => {
            diagnostics.push(node_diagnostic("action_unavailable", &message, node.id()))
        }
    }
}

pub(super) fn reachability(
    definition: &WorkflowDefinition,
    graph: &DiGraph<&str, &str>,
    indexes: &HashMap<&str, petgraph::graph::NodeIndex>,
    diagnostics: &mut Vec<WorkflowDiagnostic>,
) {
    let inputs = definition
        .nodes
        .iter()
        .filter(|node| matches!(node, WorkflowNode::Input { .. }));
    let reachable: HashSet<_> = inputs
        .flat_map(|node| petgraph::visit::Dfs::new(graph, indexes[node.id()]).iter(graph))
        .collect();
    for node in &definition.nodes {
        if !reachable.contains(&indexes[node.id()]) {
            diagnostics.push(node_diagnostic(
                "node_unreachable",
                "Workflow node is not reachable from an Input",
                node.id(),
            ));
        }
    }
}
