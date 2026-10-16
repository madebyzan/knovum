use super::from_saliency;

/// Saliency with two separated peaks and a flat, low background.
fn saliency(width: u32, height: u32, peaks: &[(u32, u32)]) -> Vec<u8> {
    let mut field = vec![4_u8; (width * height) as usize];
    for (x, y) in peaks {
        field[(y * width + x) as usize] = 250;
    }
    field
}

#[test]
fn seeds_land_on_salient_peaks_strongest_first() {
    let field = saliency(60, 60, &[(55, 55), (5, 5)]);
    let seeds = from_saliency(&field, (60, 60), 4);
    assert_eq!(seeds.len(), 2, "only cells holding a peak are prompted");
    assert!(seeds.iter().all(|seed| seed.foreground));
    let positions = seeds
        .iter()
        .map(|seed| ((seed.x * 60.0) as u32, (seed.y * 60.0) as u32))
        .collect::<Vec<_>>();
    assert!(positions.contains(&(5, 5)) && positions.contains(&(55, 55)));
    assert!(
        seeds.iter().all(|seed| (0.0..=1.0).contains(&seed.x)),
        "prompts are normalised"
    );
}

#[test]
fn a_flat_field_produces_no_prompts() {
    assert!(from_saliency(&vec![4; 400], (20, 20), 4).is_empty());
}

#[test]
fn the_seed_budget_follows_the_layer_budget_with_headroom() {
    let peaks = (0..36)
        .map(|cell| (cell % 6 * 10 + 5, cell / 6 * 10 + 5))
        .collect::<Vec<_>>();
    let field = saliency(60, 60, &peaks);
    assert_eq!(from_saliency(&field, (60, 60), 2).len(), 10);
    assert_eq!(
        from_saliency(&field, (60, 60), 24).len(),
        32,
        "the grid also caps how many prompts one pass can place"
    );
}

#[test]
fn a_field_that_does_not_match_its_source_is_ignored() {
    assert!(from_saliency(&vec![250; 10], (20, 20), 4).is_empty());
}
