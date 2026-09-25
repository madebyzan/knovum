use std::collections::HashMap;

use super::types::CapturedStep;

pub(super) fn order_steps(steps: Vec<CapturedStep>) -> Result<Vec<CapturedStep>, String> {
    fn visit(
        index: usize,
        steps: &[CapturedStep],
        by_output: &HashMap<&str, usize>,
        state: &mut [u8],
        order: &mut Vec<usize>,
    ) -> Result<(), String> {
        match state[index] {
            2 => return Ok(()),
            1 => return Err("workflow-capture-cycle: derivation inputs contain a cycle".into()),
            _ => {}
        }
        state[index] = 1;
        for input in &steps[index].inputs {
            if let Some(&dependency) = by_output.get(input.node_id.as_str()) {
                visit(dependency, steps, by_output, state, order)?;
            }
        }
        state[index] = 2;
        order.push(index);
        Ok(())
    }

    let by_output = steps
        .iter()
        .enumerate()
        .map(|(index, step)| (step.output_node_id.as_str(), index))
        .collect::<HashMap<_, _>>();
    let mut state = vec![0; steps.len()];
    let mut order = Vec::with_capacity(steps.len());
    for index in 0..steps.len() {
        visit(index, &steps, &by_output, &mut state, &mut order)?;
    }
    let mut movable = steps.into_iter().map(Some).collect::<Vec<_>>();
    Ok(order
        .into_iter()
        .map(|index| movable[index].take().expect("each step is visited once"))
        .collect())
}
