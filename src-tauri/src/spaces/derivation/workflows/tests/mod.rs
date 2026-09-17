use std::collections::{HashMap, HashSet};

use crate::spaces::knowledge::actions::{self, KnowledgeActionParameterKind};

use super::{WorkflowParameterKind, WORKFLOWS};

#[test]
fn registry_definitions_are_coherent() {
    let workflow_ids = WORKFLOWS
        .iter()
        .map(|workflow| workflow.id)
        .collect::<HashSet<_>>();
    assert_eq!(workflow_ids.len(), WORKFLOWS.len());

    for workflow in WORKFLOWS {
        assert!(workflow.version > 0);
        assert!(!workflow.steps.is_empty());
        assert_unique_steps(workflow);
        assert_known_dependencies(workflow);
        assert_acyclic(workflow);
        assert_parameter_contract(workflow);
    }
}

fn assert_acyclic(workflow: &super::WorkflowDefinition) {
    let mut remaining = workflow
        .steps
        .iter()
        .map(|step| {
            (
                step.id,
                step.depends_on.iter().copied().collect::<HashSet<_>>(),
            )
        })
        .collect::<HashMap<_, _>>();

    while !remaining.is_empty() {
        let ready = remaining
            .iter()
            .filter_map(|(id, dependencies)| dependencies.is_empty().then_some(*id))
            .collect::<Vec<_>>();
        assert!(!ready.is_empty());

        for id in ready {
            remaining.remove(id);
            for dependencies in remaining.values_mut() {
                dependencies.remove(id);
            }
        }
    }
}

fn assert_unique_steps(workflow: &super::WorkflowDefinition) {
    let step_ids = workflow
        .steps
        .iter()
        .map(|step| step.id)
        .collect::<HashSet<_>>();
    assert_eq!(step_ids.len(), workflow.steps.len());
}

fn assert_known_dependencies(workflow: &super::WorkflowDefinition) {
    let step_ids = workflow
        .steps
        .iter()
        .map(|step| step.id)
        .collect::<HashSet<_>>();

    for step in workflow.steps {
        assert!(!step.depends_on.contains(&step.id));
        assert!(step
            .depends_on
            .iter()
            .all(|dependency| step_ids.contains(dependency)));
    }
}

fn assert_parameter_contract(workflow: &super::WorkflowDefinition) {
    let mut required = HashMap::new();
    for step in workflow.steps {
        let action = actions::require(step.action).expect("registered Knowledge Action");
        for parameter in action.parameters {
            required.insert(parameter.key, action_parameter_kind(parameter.kind));
        }
    }

    let declared = workflow
        .parameters
        .iter()
        .map(|parameter| (parameter.key, parameter.kind))
        .collect::<HashMap<_, _>>();
    assert_eq!(required, declared);
}

fn action_parameter_kind(kind: KnowledgeActionParameterKind) -> WorkflowParameterKind {
    match kind {
        KnowledgeActionParameterKind::Text => WorkflowParameterKind::Text,
        KnowledgeActionParameterKind::Number => WorkflowParameterKind::Number,
    }
}
