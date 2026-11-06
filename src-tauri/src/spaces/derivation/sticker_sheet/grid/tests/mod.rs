use super::cells;

const PITCH: u32 = 100;

/// Filled rectangles `(left, top, width, height)` on a blank mask.
fn paint(
    mask: &mut [bool],
    width: u32,
    (left, top, block_width, block_height): (u32, u32, u32, u32),
) {
    for y in top..top + block_height {
        for x in left..left + block_width {
            mask[(y * width + x) as usize] = true;
        }
    }
}

/// A sheet of `PITCH` cells, one entry of `rows` per row giving how many stickers it holds. Each
/// sticker is a character with a three-glyph caption a short gap under it; the gap under the
/// character is much narrower than the gutter between two stickers.
fn sheet(rows: &[u32], columns: u32) -> (Vec<bool>, (u32, u32)) {
    let (width, height) = (columns * PITCH, rows.len() as u32 * PITCH);
    let mut mask = vec![false; (width * height) as usize];
    for (row, count) in rows.iter().enumerate() {
        for column in 0..*count {
            let (left, top) = (column * PITCH, row as u32 * PITCH);
            paint(&mut mask, width, (left + 25, top + 15, 50, 40));
            for glyph in 0..3 {
                paint(&mut mask, width, (left + 30 + glyph * 14, top + 61, 9, 8));
            }
        }
    }
    (mask, (width, height))
}

#[test]
fn every_cell_of_a_grid_is_one_sticker_with_its_caption() {
    let (mask, dimensions) = sheet(&[4, 4, 4], 4);
    let found = cells(&mask, dimensions);
    assert_eq!(found.len(), 12);
    assert_eq!(
        found[0],
        [25, 15, 74, 68],
        "the character and the caption under it"
    );
    assert_eq!(
        found[5],
        [125, 115, 174, 168],
        "reading order: the second row's second cell"
    );
}

#[test]
fn a_row_holding_fewer_stickers_is_read_as_it_is() {
    let (mask, dimensions) = sheet(&[3, 3, 2], 3);
    let found = cells(&mask, dimensions);
    assert_eq!(found.len(), 8);
    assert_eq!(found[7][0], 125, "the last row ends at its second sticker");
}

#[test]
fn a_caption_gap_is_not_read_as_a_row_of_its_own() {
    // An even pitch through the caption gaps lands in the characters of the next row, and a
    // pitch that cuts through some of them leaves a caption-only sliver; only the true rows
    // survive.
    let (mask, dimensions) = sheet(&[2, 2, 2], 2);
    let found = cells(&mask, dimensions);
    assert_eq!(found.len(), 6);
    assert!(found.iter().all(|[_, top, _, bottom]| bottom - top == 53));
}

#[test]
fn stickers_drawn_at_different_widths_are_still_one_cell_each() {
    // The gutters are 30, 20 and 8 pixels wide: they follow how wide each neighbour is drawn,
    // while the cells stay evenly pitched.
    let (width, height) = (400, PITCH);
    let mut mask = vec![false; (width * height) as usize];
    for (left, right) in [(10, 70), (100, 175), (195, 292), (300, 380)] {
        paint(&mut mask, width, (left, 25, right - left, 50));
    }
    let found = cells(&mask, (width, height));
    assert_eq!(
        found.iter().map(|[left, ..]| *left).collect::<Vec<_>>(),
        [10, 100, 195, 300]
    );
}

#[test]
fn a_short_row_set_off_centre_is_read_as_it_is() {
    let (mut mask, (width, height)) = sheet(&[3], 3);
    let mut second = vec![false; (width * PITCH) as usize];
    for left in [75, 175] {
        paint(&mut second, width, (left, 15, 50, 40));
    }
    mask.extend(second);
    let found = cells(&mask, (width, height + PITCH));
    assert_eq!(
        found.iter().map(|[left, ..]| *left).collect::<Vec<_>>(),
        [25, 125, 225, 75, 175]
    );
}

#[test]
fn a_sticker_reaching_past_its_cell_edge_stays_one_cell() {
    // No exact lattice fits: an even pitch of 100 puts the first edge at 100, inside the first
    // sticker. Within the overhang a sheet laid out by eye allows, it is four cells.
    let (width, height) = (400, PITCH);
    let mut mask = vec![false; (width * height) as usize];
    for (left, right) in [(0, 104), (106, 196), (204, 296), (298, 400)] {
        paint(&mut mask, width, (left, 25, right - left, 50));
    }
    let found = cells(&mask, (width, height));
    assert_eq!(
        found.iter().map(|[left, ..]| *left).collect::<Vec<_>>(),
        [0, 106, 204, 298]
    );
}

#[test]
fn a_prop_beside_a_character_stays_with_it() {
    // A lattice of twice the cells cuts every prop off its character; a prop alone is not
    // comparable in size to a sticker.
    let (width, height) = (3 * PITCH, PITCH);
    let mut mask = vec![false; (width * height) as usize];
    for column in 0..3 {
        let left = column * PITCH;
        paint(&mut mask, width, (left + 10, 20, 50, 60));
        paint(&mut mask, width, (left + 66, 40, 14, 20));
    }
    let found = cells(&mask, (width, height));
    assert_eq!(
        found,
        [[10, 20, 79, 79], [110, 20, 179, 79], [210, 20, 279, 79]]
    );
}

#[test]
fn one_sticker_is_one_cell_and_a_blank_sheet_has_none() {
    let (mask, dimensions) = sheet(&[1], 1);
    assert_eq!(cells(&mask, dimensions), [[25, 15, 74, 68]]);
    assert!(cells(&vec![false; 400], (20, 20)).is_empty());
}

#[test]
fn a_stray_dot_in_a_gutter_does_not_close_it() {
    let (mut mask, dimensions) = sheet(&[3, 3, 3], 3);
    mask[(85 * dimensions.0 + 150) as usize] = true;
    assert_eq!(cells(&mask, dimensions).len(), 9);
}
