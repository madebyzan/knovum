use std::collections::{BTreeMap, BTreeSet};

use super::{TemplateDependency, WorkflowDefinition, WorkflowNode};

pub(super) fn order_dependencies(
    dependencies: &[TemplateDependency],
) -> Result<Vec<&TemplateDependency>, String> {
    let available = dependencies
        .iter()
        .map(|item| (item.workflow_id.as_str(), item.version))
        .collect::<BTreeSet<_>>();
    let mut resolved = BTreeSet::new();
    let mut ordered = Vec::with_capacity(dependencies.len());
    while ordered.len() < dependencies.len() {
        let before = ordered.len();
        for dependency in dependencies {
            let key = (dependency.workflow_id.as_str(), dependency.version);
            if resolved.contains(&key) {
                continue;
            }
            let ready = references(&dependency.definition).iter().all(|item| {
                !available.contains(&(item.0.as_str(), item.1))
                    || resolved.contains(&(item.0.as_str(), item.1))
            });
            if ready {
                resolved.insert(key);
                ordered.push(dependency);
            }
        }
        if ordered.len() == before {
            return Err(
                "workflow-template-dependency-cycle: dependency graph contains a cycle".into(),
            );
        }
    }
    Ok(ordered)
}

pub(super) fn rewrite_references(
    mut definition: WorkflowDefinition,
    mappings: &BTreeMap<(String, u32), String>,
) -> Result<WorkflowDefinition, String> {
    for node in &mut definition.nodes {
        let WorkflowNode::Subworkflow {
            workflow_id,
            version,
            ..
        } = node
        else {
            continue;
        };
        let replacement = mappings
            .get(&(workflow_id.clone(), *version))
            .ok_or("workflow-template-dependency-incomplete: referenced version is not bundled")?;
        *workflow_id = replacement.clone();
        *version = 1;
    }
    Ok(definition)
}

pub(super) fn references(definition: &WorkflowDefinition) -> Vec<(String, u32)> {
    definition
        .nodes
        .iter()
        .filter_map(|node| match node {
            WorkflowNode::Subworkflow {
                workflow_id,
                version,
                ..
            } => Some((workflow_id.clone(), *version)),
            _ => None,
        })
        .collect()
}

pub(super) fn reject_external_tools(definition: &WorkflowDefinition) -> Result<(), String> {
    if definition
        .nodes
        .iter()
        .any(|node| matches!(node, WorkflowNode::ExternalTool { .. }))
    {
        return Err("workflow-template-external-tool-not-portable: Space-owned bindings cannot enter a personal template".into());
    }
    Ok(())
}
