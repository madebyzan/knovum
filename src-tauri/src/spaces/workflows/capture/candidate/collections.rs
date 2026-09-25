use std::collections::{HashMap, HashSet};

use super::super::types::{CandidateSource, CaptureCandidate};
use crate::spaces::actions;

pub(super) fn group(candidate: &mut CaptureCandidate) -> Result<(), &'static str> {
    let uses = candidate
        .steps
        .iter()
        .flat_map(|step| &step.inputs)
        .filter_map(|input| match &input.source {
            CandidateSource::InputSlot { input_slot_id } => Some(input_slot_id.clone()),
            _ => None,
        })
        .fold(HashMap::<String, usize>::new(), |mut counts, id| {
            *counts.entry(id).or_default() += 1;
            counts
        });
    let slot_kinds = candidate
        .input_slots
        .iter()
        .map(|slot| (slot.id.clone(), slot.kind.clone()))
        .collect::<HashMap<_, _>>();
    let mut removed = HashSet::new();
    for step in &mut candidate.steps {
        let Some(contract) = actions::require(&step.action_id)
            .map_err(|_| "candidate-binding-unavailable")?
            .collection_input
        else {
            continue;
        };
        let members = step
            .inputs
            .iter()
            .enumerate()
            .filter_map(|(index, input)| match &input.source {
                CandidateSource::InputSlot { input_slot_id }
                    if input.role == contract.role
                        && slot_kinds.get(input_slot_id).map(String::as_str)
                            == Some(contract.kind)
                        && uses.get(input_slot_id.as_str()) == Some(&1) =>
                {
                    Some((index, input_slot_id.clone(), input.revision_basis.clone()))
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        if members.is_empty() {
            continue;
        }
        if members.len() > contract.max_items
            || members.iter().any(|member| member.2 != members[0].2)
        {
            return Err("candidate-binding-unavailable");
        }
        let first_id = members[0].1.clone();
        let first_position = members[0].0;
        step.inputs[first_position].source = CandidateSource::InputSequence {
            input_slot_id: first_id.clone(),
        };
        let later = members
            .iter()
            .skip(1)
            .map(|member| member.0)
            .collect::<HashSet<_>>();
        step.inputs = step
            .inputs
            .drain(..)
            .enumerate()
            .filter_map(|(index, binding)| (!later.contains(&index)).then_some(binding))
            .collect();
        for (position, binding) in step.inputs.iter_mut().enumerate() {
            binding.position = position as i64;
        }
        if let Some(slot) = candidate
            .input_slots
            .iter_mut()
            .find(|slot| slot.id == first_id)
        {
            slot.max_items = contract.max_items;
            slot.ordered = true;
        }
        removed.extend(members.into_iter().skip(1).map(|member| member.1));
    }
    candidate
        .input_slots
        .retain(|slot| !removed.contains(&slot.id));
    Ok(())
}
