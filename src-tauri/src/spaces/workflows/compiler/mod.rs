mod diagnostics;
mod ports;
mod validation;

use std::collections::HashMap;

use petgraph::{algo::toposort, graph::DiGraph};
use serde::Serialize;

use super::definition::WorkflowDefinition;
pub use diagnostics::WorkflowDiagnostic;
use diagnostics::{edge_diagnostic, node_diagnostic};

pub const COMPILER_VERSION: u16 = 1;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowExecutionPlan {
    pub compiler_version: u16,
    pub ordered_node_ids: Vec<String>,
    pub dependency_digest: String,
}

pub fn compile(
    definition: &WorkflowDefinition,
) -> Result<WorkflowExecutionPlan, Vec<WorkflowDiagnostic>> {
    let mut diagnostics = validation::definition(definition);
    let mut graph = DiGraph::<&str, &str>::new();
    let mut indexes = HashMap::new();
    for node in &definition.nodes {
        indexes.insert(node.id(), graph.add_node(node.id()));
    }
    for edge in &definition.edges {
        match (
            indexes.get(edge.source_node_id.as_str()),
            indexes.get(edge.target_node_id.as_str()),
        ) {
            (Some(source), Some(target)) => {
                graph.add_edge(*source, *target, &edge.id);
            }
            _ => diagnostics.push(edge_diagnostic(
                "edge_endpoint_missing",
                "Workflow edge references an unavailable node",
                &edge.id,
            )),
        }
    }
    ports::validate(definition, &mut diagnostics);
    let ordered = match toposort(&graph, None) {
        Ok(indexes) => indexes
            .into_iter()
            .map(|index| graph[index].to_owned())
            .collect(),
        Err(cycle) => {
            diagnostics.push(node_diagnostic(
                "cycle_forbidden",
                "Workflow graph contains a cycle; use Iterate for bounded repetition",
                graph[cycle.node_id()],
            ));
            Vec::new()
        }
    };
    validation::reachability(definition, &graph, &indexes, &mut diagnostics);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    Ok(WorkflowExecutionPlan {
        compiler_version: COMPILER_VERSION,
        ordered_node_ids: ordered,
        dependency_digest: dependency_digest(definition),
    })
}

fn dependency_digest(definition: &WorkflowDefinition) -> String {
    use sha2::Digest;
    let mut dependencies = definition
        .nodes
        .iter()
        .filter_map(|node| match node {
            super::definition::WorkflowNode::Subworkflow {
                workflow_id,
                version,
                ..
            } => Some(format!("{workflow_id}:{version}")),
            _ => None,
        })
        .collect::<Vec<_>>();
    dependencies.sort();
    format!("{:x}", sha2::Sha256::digest(dependencies.join("\n")))
}
