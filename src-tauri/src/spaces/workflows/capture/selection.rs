use std::collections::{HashMap, HashSet};

use super::types::CapturedStep;

pub(super) fn groups(selected: &HashSet<String>, steps: &[CapturedStep]) -> Vec<Vec<String>> {
    let mut adjacent: HashMap<&str, Vec<&str>> = HashMap::new();
    for step in steps {
        for input in &step.inputs {
            adjacent
                .entry(&step.output_node_id)
                .or_default()
                .push(&input.node_id);
            adjacent
                .entry(&input.node_id)
                .or_default()
                .push(&step.output_node_id);
        }
    }
    let mut ordered = selected.iter().map(String::as_str).collect::<Vec<_>>();
    ordered.sort_unstable();
    let mut visited = HashSet::new();
    let mut groups = Vec::new();
    for origin in ordered {
        if !visited.insert(origin) {
            continue;
        }
        let mut pending = vec![origin];
        let mut group = Vec::new();
        while let Some(node_id) = pending.pop() {
            if selected.contains(node_id) {
                group.push(node_id.to_owned());
            }
            for neighbor in adjacent.get(node_id).into_iter().flatten() {
                if visited.insert(neighbor) {
                    pending.push(neighbor);
                }
            }
        }
        group.sort();
        groups.push(group);
    }
    groups
}
