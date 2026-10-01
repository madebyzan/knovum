use std::collections::{HashMap, HashSet};

use super::types::{BoundaryInput, BoundaryUse, CapturedStep};

pub(super) fn insert(
    boundaries: &mut HashMap<String, BoundaryInput>,
    node_id: String,
    kind: String,
    selected: &HashSet<String>,
) {
    boundaries.insert(
        node_id.clone(),
        BoundaryInput {
            selected: selected.contains(&node_id),
            node_id,
            kind,
            uses: Vec::new(),
        },
    );
}

pub(super) fn finish(
    mut boundaries: HashMap<String, BoundaryInput>,
    steps: &[CapturedStep],
) -> Vec<BoundaryInput> {
    for step in steps {
        for input in &step.inputs {
            if let Some(boundary) = boundaries.get_mut(&input.node_id) {
                boundary.uses.push(BoundaryUse {
                    derivation_id: step.derivation_id.clone(),
                    position: input.position,
                    role: input.role,
                    source_revision: input.source_revision.clone(),
                    revision_basis: input.revision_basis,
                });
            }
        }
    }
    let mut inputs = boundaries.into_values().collect::<Vec<_>>();
    inputs.sort_by(|left, right| left.node_id.cmp(&right.node_id));
    inputs
}
