use std::collections::{BTreeMap, HashMap};

use super::*;
use crate::spaces::workflows::definition::{FailurePolicy, WorkflowEdge, WorkflowLimits};

#[test]
fn dependencies_are_ordered_children_first_and_rewritten() {
    let leaf = dependency("leaf", 4, definition(Vec::new()));
    let parent = dependency(
        "parent",
        2,
        definition(vec![subworkflow("child", "leaf", 4)]),
    );
    let dependencies = [parent.clone(), leaf.clone()];
    let ordered = order_dependencies(&dependencies).expect("valid closure");
    assert_eq!(
        ordered
            .iter()
            .map(|item| item.workflow_id.as_str())
            .collect::<Vec<_>>(),
        ["leaf", "parent"]
    );

    let mappings = BTreeMap::from([(("leaf".into(), 4), "copied-leaf".into())]);
    let rewritten = rewrite_references(parent.definition, &mappings).expect("complete closure");
    let WorkflowNode::Subworkflow {
        workflow_id,
        version,
        ..
    } = &rewritten.nodes[0]
    else {
        panic!("expected subworkflow")
    };
    assert_eq!((workflow_id.as_str(), *version), ("copied-leaf", 1));
}

#[test]
fn external_tool_bindings_are_not_portable_authority() {
    let workflow = definition(vec![WorkflowNode::ExternalTool {
        id: "tool".into(),
        binding_id: "space-binding".into(),
        schema_fingerprint: "f".repeat(64),
    }]);
    assert!(reject_external_tools(&workflow).is_err());
}

fn dependency(
    workflow_id: &str,
    version: u32,
    definition: WorkflowDefinition,
) -> TemplateDependency {
    TemplateDependency {
        workflow_id: workflow_id.into(),
        version,
        definition,
    }
}

fn subworkflow(id: &str, workflow_id: &str, version: u32) -> WorkflowNode {
    WorkflowNode::Subworkflow {
        id: id.into(),
        workflow_id: workflow_id.into(),
        version,
    }
}

fn definition(nodes: Vec<WorkflowNode>) -> WorkflowDefinition {
    WorkflowDefinition {
        schema_version: 1,
        title: "Template".into(),
        nodes,
        edges: Vec::<WorkflowEdge>::new(),
        layout: HashMap::new(),
        failure_policy: FailurePolicy::StopRun,
        limits: WorkflowLimits {
            max_expanded_steps: 100,
            max_parallelism: 4,
            max_duration_seconds: 300,
            max_total_tokens: 10_000,
            max_step_attempts: 3,
            max_input_bytes: 1_000_000,
            max_output_bytes: 1_000_000,
        },
    }
}
