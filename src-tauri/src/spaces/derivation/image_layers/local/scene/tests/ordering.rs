use super::super::passes::Field;
use super::{salient_layer, stack};

fn field(values: Vec<f32>, width: u32, height: u32) -> Field {
    Field {
        values,
        width,
        height,
    }
}

#[test]
fn subjects_are_stacked_from_far_to_near() {
    // Rank 1 is the nearest subject and rank 2 the furthest, so the stack must invert them.
    let ranks = vec![1, 1, 2, 2];
    let depth = field(vec![0.9, 0.9, 0.2, 0.2], 4, 1);
    let decomposition = stack(ranks, (4, 1), &depth).expect("stack");
    assert_eq!(decomposition.count, 2);
    assert_eq!(
        decomposition.labels,
        [2, 2, 1, 1],
        "the far subject becomes the lower layer"
    );
}

#[test]
fn a_depth_field_at_its_own_resolution_is_sampled_rather_than_upsampled() {
    let ranks = vec![0, 0, 1, 1, 0, 0, 1, 1];
    let depth = field(vec![0.1, 0.8], 2, 1);
    let decomposition = stack(ranks, (4, 2), &depth).expect("stack");
    assert_eq!(decomposition.count, 1);
    assert_eq!(decomposition.labels, [0, 0, 1, 1, 0, 0, 1, 1]);
}

#[test]
fn a_mismatched_depth_field_is_rejected() {
    assert!(stack(vec![0, 1], (2, 1), &field(Vec::new(), 0, 0)).is_err());
    assert!(stack(vec![0, 1, 1], (2, 1), &field(vec![0.1, 0.2], 2, 1)).is_err());
}

#[test]
fn the_layer_holding_the_salient_subject_owns_the_soft_alpha() {
    let decomposition = stack(
        vec![1, 1, 2, 2],
        (4, 1),
        &field(vec![0.1, 0.1, 0.9, 0.9], 4, 1),
    )
    .expect("stack");
    let (owner, alpha) = salient_layer(&decomposition, vec![250, 250, 2, 2]).expect("owner");
    assert_eq!(
        owner, decomposition.labels[0],
        "the saliency peak sits on the first subject"
    );
    assert_eq!(alpha.len(), 4);
    assert!(
        salient_layer(&decomposition, vec![1, 2]).is_none(),
        "shape must match"
    );
}
