use super::{elect, probe_size, Candidate};

fn candidate(seed: usize, rank: f32, mask: [bool; 8]) -> Candidate {
    Candidate {
        seed,
        mask: mask.to_vec(),
        area: mask.iter().filter(|inside| **inside).count(),
        rank,
    }
}

fn seeds(kept: &[Candidate]) -> Vec<usize> {
    kept.iter().map(|entry| entry.seed).collect()
}

#[test]
fn probing_scales_the_long_edge_and_leaves_small_sources_untouched() {
    assert_eq!(probe_size(1080, 1920), (108, 192));
    assert_eq!(probe_size(1920, 1080), (192, 108));
    assert_eq!(probe_size(100, 160), (100, 160));
    let (height, width) = probe_size(4000, 2);
    assert!(
        height == 192 && width >= 1,
        "narrow sources keep one column"
    );
}

#[test]
fn election_drops_candidates_that_repeat_or_subdivide_a_stronger_one() {
    let kept = elect(
        vec![
            candidate(0, 0.9, [true, true, true, true, false, false, false, false]),
            candidate(
                1,
                0.8,
                [true, true, true, false, false, false, false, false],
            ),
            candidate(
                2,
                0.7,
                [true, false, false, false, false, false, false, false],
            ),
            candidate(
                3,
                0.6,
                [false, false, false, false, true, true, false, false],
            ),
        ],
        8,
    );
    assert_eq!(
        seeds(&kept),
        [0, 3],
        "the duplicate and the part belong to the elected subject, the neighbor does not"
    );
}

#[test]
fn election_honors_the_layer_budget_in_rank_order() {
    let kept = elect(
        vec![
            candidate(
                0,
                0.9,
                [true, false, false, false, false, false, false, false],
            ),
            candidate(
                1,
                0.8,
                [false, true, false, false, false, false, false, false],
            ),
            candidate(
                2,
                0.7,
                [false, false, true, false, false, false, false, false],
            ),
        ],
        2,
    );
    assert_eq!(seeds(&kept), [0, 1]);
}
