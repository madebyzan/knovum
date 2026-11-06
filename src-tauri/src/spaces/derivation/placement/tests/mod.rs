use serde_json::json;

use super::{Grid, SetMember, CONTENT_GAP, MEMBER_FLOOR, PLACEHOLDER};
use crate::spaces::structure::{Size, SizingMode};

fn picture(width: u32, height: u32) -> SetMember {
    SetMember::new(
        &json!({
            "kind": "image", "title": "layer",
            "intrinsicWidth": width, "intrinsicHeight": height
        }),
        &PLACEHOLDER,
        SizingMode::Content,
    )
}

fn document() -> SetMember {
    SetMember::new(
        &json!({ "kind": "markdown", "title": "notes" }),
        &Size {
            width: 260.0,
            height: 184.0,
        },
        SizingMode::Frame,
    )
}

#[test]
fn a_small_set_keeps_each_output_at_the_size_it_would_have_alone() {
    let grid = Grid::new(vec![picture(1600, 1600); 4]).expect("grid");
    let cell = &grid.cells[3];
    assert_eq!(
        (cell.member.size.width, cell.member.size.height),
        (280.0, 280.0)
    );
    assert_eq!(cell.member.sizing_mode, SizingMode::Content);
    assert_eq!((cell.x, cell.y), (280.0 + CONTENT_GAP, 280.0 + CONTENT_GAP));
    assert_eq!(grid.width, 2.0 * 280.0 + CONTENT_GAP);
}

#[test]
fn a_larger_set_keeps_the_area_of_four_outputs() {
    // Nine pictures shrink by √(4/9): pictures and gutters alike, so the set reads as one.
    let grid = Grid::new(vec![picture(1600, 1600); 9]).expect("grid");
    let member = &grid.cells[0].member;
    assert_eq!((member.size.width, member.size.height), (187.0, 187.0));
    assert_eq!(
        member.sizing_mode,
        SizingMode::Frame,
        "the set chose this footprint, so settling content keeps it"
    );
    let gap = (CONTENT_GAP * 2.0 / 3.0).round();
    assert_eq!(grid.width, 3.0 * 187.0 + 2.0 * gap);
    assert_eq!(
        (grid.cells[3].x, grid.cells[3].y),
        (0.0, 187.0 + gap),
        "the fourth output starts the second row"
    );
}

#[test]
fn a_sticker_sheet_reads_at_one_glance() {
    let grid = Grid::new(vec![picture(240, 240); 24]).expect("grid");
    let member = &grid.cells[0].member;
    assert!(member.size.width < 120.0 && member.size.width >= MEMBER_FLOOR);
    assert!(
        grid.width < 3.0 * 280.0,
        "five columns span less than three lone outputs"
    );
}

#[test]
fn a_picture_never_shrinks_below_the_floor_and_a_small_one_keeps_its_own_size() {
    let grid = Grid::new(vec![picture(1600, 800); 64]).expect("grid");
    assert_eq!(grid.cells[0].member.size.width, MEMBER_FLOOR);
    let icons = Grid::new(vec![picture(64, 64); 64]).expect("grid");
    assert_eq!(icons.cells[0].member.size.width, 64.0);
    assert_eq!(icons.cells[0].member.sizing_mode, SizingMode::Content);
}

#[test]
fn only_pictures_shrink_in_a_set() {
    let mut members = vec![picture(1600, 1600); 8];
    members.push(document());
    let grid = Grid::new(members).expect("grid");
    let notes = &grid.cells[8].member;
    assert_eq!((notes.size.width, notes.size.height), (260.0, 184.0));
    assert_eq!(notes.sizing_mode, SizingMode::Frame);
}

#[test]
fn a_short_last_row_does_not_claim_a_row_of_its_own() {
    let grid = Grid::new(vec![picture(400, 400); 8]).expect("grid");
    let size = grid.cells[0].member.size.clone();
    let gap = (CONTENT_GAP * (4.0_f64 / 8.0).sqrt()).round();
    assert_eq!(grid.height, 3.0 * size.height + 2.0 * gap);
    assert_eq!(grid.cells[7].y, 2.0 * (size.height + gap));
    assert_eq!(grid.cells[7].x, size.width + gap);
}

#[test]
fn each_output_keeps_its_own_aspect_and_is_centred_in_its_cell() {
    let wide = picture(1600, 400);
    let tall = picture(400, 1600);
    let grid = Grid::new(vec![wide.clone(), tall.clone()]).expect("grid");
    assert_eq!(grid.cells[0].member.size, wide.size);
    assert_eq!(
        grid.width,
        2.0 * wide.size.width + CONTENT_GAP,
        "cells are equal"
    );
    assert_eq!(grid.height, tall.size.height);
    assert_eq!(grid.cells[0].y, (tall.size.height - wide.size.height) / 2.0);
    assert_eq!(
        grid.cells[1].x,
        wide.size.width + CONTENT_GAP + (wide.size.width - tall.size.width) / 2.0
    );
}

#[test]
fn content_that_cannot_report_a_size_keeps_the_placeholder_footprint() {
    let pending = SetMember::new(
        &json!({ "kind": "image", "title": "pending" }),
        &PLACEHOLDER,
        SizingMode::Content,
    );
    let grid = Grid::new(vec![pending]).expect("grid");
    assert_eq!(grid.width, PLACEHOLDER.width);
    assert_eq!(grid.height, PLACEHOLDER.height);
}

#[test]
fn a_derivation_publishes_at_least_one_output() {
    assert!(Grid::new(Vec::new()).is_err());
}
