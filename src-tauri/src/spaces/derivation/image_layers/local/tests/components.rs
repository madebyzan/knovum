use super::{find, merge, reading_order};

/// Two separated blocks per row over three rows, written in column-major order so the fixture
/// cannot pass by accident when ordering is skipped.
fn grid(width: u32, height: u32, blocks: &[(u32, u32, u32, u32)]) -> Vec<bool> {
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

#[test]
fn components_are_found_largest_first_and_tiny_specks_are_dropped() {
    let mask = grid(40, 40, &[(0, 0, 10, 10), (20, 20, 6, 6), (38, 0, 1, 1)]);
    let found = find(&mask, 40, 40);
    assert_eq!(found.len(), 2, "the single pixel speck is not a subject");
    assert_eq!(found[0].pixels.len(), 100);
    assert_eq!(found[0].bounds, [0, 0, 9, 9]);
    assert_eq!(found[1].pixels.len(), 36);
}

#[test]
fn diagonally_touching_pixels_belong_to_one_component() {
    let mut mask = grid(20, 20, &[(2, 2, 4, 4)]);
    mask[(6 * 20 + 6) as usize] = true;
    let found = find(&mask, 20, 20);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].bounds, [2, 2, 6, 6]);
}

#[test]
fn a_part_inside_a_subject_bounding_box_is_merged_into_it() {
    let mut mask = grid(60, 60, &[(5, 5, 40, 6), (5, 39, 40, 6)]);
    for y in 20..32 {
        for x in 20..32 {
            mask[(y * 60 + x) as usize] = true;
        }
    }
    let found = find(&mask, 60, 60);
    assert_eq!(found.len(), 3, "two bars and one inner block");
    let merged = merge(found);
    assert_eq!(
        merged.len(),
        2,
        "the inner block sits inside no single bar bounding box and stays separate"
    );
}

#[test]
fn reading_order_walks_rows_left_to_right() {
    let mask = grid(
        300,
        300,
        &[
            (200, 10, 40, 40),
            (10, 10, 40, 40),
            (110, 10, 40, 40),
            (10, 200, 40, 40),
        ],
    );
    let ordered = reading_order(find(&mask, 300, 300));
    assert_eq!(
        ordered
            .iter()
            .map(|component| (component.bounds[0], component.bounds[1]))
            .collect::<Vec<_>>(),
        [(10, 10), (110, 10), (200, 10), (10, 200)]
    );
}
