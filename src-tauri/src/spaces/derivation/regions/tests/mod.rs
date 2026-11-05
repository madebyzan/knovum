use super::{connected, dilate, erode, merge_contained, reachable_from_edge};

/// Filled rectangles `(left, top, width, height)` on an empty mask.
fn mask(width: u32, height: u32, blocks: &[(u32, u32, u32, u32)]) -> Vec<bool> {
    let mut mask = vec![false; (width * height) as usize];
    for (left, top, block_width, block_height) in blocks {
        for y in *top..top + block_height {
            for x in *left..left + block_width {
                mask[(y * width + x) as usize] = true;
            }
        }
    }
    mask
}

fn count(mask: &[bool]) -> usize {
    mask.iter().filter(|inside| **inside).count()
}

#[test]
fn regions_are_found_largest_first() {
    let mask = mask(40, 40, &[(20, 20, 6, 6), (0, 0, 10, 10), (38, 0, 1, 1)]);
    let found = connected(&mask, (40, 40));
    assert_eq!(
        found
            .iter()
            .map(|region| region.pixels.len())
            .collect::<Vec<_>>(),
        [100, 36, 1]
    );
    assert_eq!(found[0].bounds, [0, 0, 9, 9]);
}

#[test]
fn diagonally_touching_pixels_belong_to_one_region() {
    let mut mask = mask(20, 20, &[(2, 2, 4, 4)]);
    mask[(6 * 20 + 6) as usize] = true;
    let found = connected(&mask, (20, 20));
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].bounds, [2, 2, 6, 6]);
}

#[test]
fn a_detached_part_inside_a_subject_merges_into_that_subject() {
    // A hollow frame with a detached block in its middle: one subject, not two.
    let frame = [(5, 5, 40, 4), (5, 41, 40, 4), (5, 5, 4, 40), (41, 5, 4, 40)];
    let mut blocks = frame.to_vec();
    blocks.push((20, 20, 12, 12));
    let found = connected(&mask(60, 60, &blocks), (60, 60));
    assert_eq!(found.len(), 2, "the frame and the block it encloses");
    let total = found
        .iter()
        .map(|region| region.pixels.len())
        .sum::<usize>();
    let merged = merge_contained(found);
    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].pixels.len(), total);
    assert_eq!(merged[0].bounds, [5, 5, 44, 44]);
}

#[test]
fn regions_that_only_sit_near_each_other_stay_separate() {
    let found = connected(&mask(60, 60, &[(5, 5, 40, 6), (5, 39, 40, 6)]), (60, 60));
    assert_eq!(merge_contained(found).len(), 2);
}

#[test]
fn only_what_the_edge_reaches_is_reachable() {
    // A closed ring: its inside is passable but sealed off from the edge.
    let ring = mask(
        20,
        20,
        &[(4, 4, 12, 2), (4, 14, 12, 2), (4, 4, 2, 12), (14, 4, 2, 12)],
    );
    let passable = ring.iter().map(|wall| !wall).collect::<Vec<_>>();
    let reached = reachable_from_edge(&passable, (20, 20));
    assert!(reached[0], "the corner is on the edge");
    assert!(
        !reached[(10 * 20 + 10) as usize],
        "the sealed inside is not reached"
    );
    assert_eq!(
        count(&reached),
        400 - 144,
        "everything outside the ring is reached"
    );
}

#[test]
fn a_diagonal_gap_does_not_open_an_outline() {
    // A diamond drawn in diagonal steps: 8-connected the inside would leak out between two wall
    // pixels that only touch at a corner; 4-connected the outline is closed.
    let wall_at = [
        (3, 1),
        (4, 2),
        (5, 3),
        (4, 4),
        (3, 5),
        (2, 4),
        (1, 3),
        (2, 2),
    ];
    let mut passable = vec![true; 49];
    for (x, y) in wall_at {
        passable[y * 7 + x] = false;
    }
    let reached = reachable_from_edge(&passable, (7, 7));
    assert!(reached[0], "the outside reaches the corner");
    for (x, y) in [(3, 3), (3, 2), (4, 3), (3, 4), (2, 3)] {
        assert!(!reached[y * 7 + x], "({x}, {y}) is inside the diamond");
    }
}

#[test]
fn dilation_grows_by_a_square_radius_and_erosion_undoes_it() {
    let block = mask(30, 30, &[(10, 10, 6, 6)]);
    let grown = dilate(&block, (30, 30), 2);
    assert_eq!(count(&grown), 10 * 10);
    assert!(grown[(8 * 30 + 8) as usize] && !grown[(7 * 30 + 8) as usize]);
    assert_eq!(erode(&grown, (30, 30), 2), block);
    assert_eq!(dilate(&block, (30, 30), 0), block);
}

#[test]
fn the_canvas_edge_does_not_erode_a_subject() {
    let filled = vec![true; 12];
    assert_eq!(erode(&filled, (4, 3), 1), filled);
    let mut notched = filled.clone();
    notched[5] = false;
    assert_eq!(
        count(&erode(&notched, (4, 3), 1)),
        12 - 9,
        "only the notch erodes its neighbourhood"
    );
}
