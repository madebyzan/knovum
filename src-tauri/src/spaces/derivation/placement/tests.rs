use serde_json::json;

use super::{fitted, Grid, PLACEHOLDER};
use crate::spaces::structure::{Size, SizingMode, CONTENT_GAP};

fn layer(width: u32, height: u32) -> Size {
    fitted(
        &json!({
            "kind": "image", "title": "layer",
            "intrinsicWidth": width, "intrinsicHeight": height
        }),
        &PLACEHOLDER,
        SizingMode::Content,
    )
}

#[test]
fn a_set_of_outputs_is_laid_out_as_a_grid_rather_than_one_row() {
    let cell = layer(400, 400);
    let grid = Grid::new(vec![cell.clone(); 9]).expect("grid");
    assert_eq!(grid.width, 3.0 * cell.width + 2.0 * CONTENT_GAP);
    assert_eq!(grid.height, 3.0 * cell.height + 2.0 * CONTENT_GAP);
    assert_eq!(grid.cells[2].x, 2.0 * (cell.width + CONTENT_GAP));
    assert_eq!(grid.cells[2].y, 0.0);
    assert_eq!(
        (grid.cells[3].x, grid.cells[3].y),
        (0.0, cell.height + CONTENT_GAP),
        "the fourth output starts the second row"
    );
}

#[test]
fn a_short_last_row_does_not_claim_a_row_of_its_own() {
    let cell = layer(400, 400);
    let grid = Grid::new(vec![cell.clone(); 8]).expect("grid");
    assert_eq!(grid.height, 3.0 * cell.height + 2.0 * CONTENT_GAP);
    assert_eq!(grid.cells[7].y, 2.0 * (cell.height + CONTENT_GAP));
    assert_eq!(grid.cells[7].x, cell.width + CONTENT_GAP);
}

#[test]
fn each_output_keeps_its_own_size_and_is_centred_in_its_cell() {
    let wide = layer(1600, 400);
    let tall = layer(400, 1600);
    assert!(
        wide.width > tall.width && tall.height > wide.height,
        "the two layers are sized by their own aspect"
    );
    let grid = Grid::new(vec![wide.clone(), tall.clone()]).expect("grid");
    assert_eq!(grid.cells[0].size.width, wide.width);
    assert_eq!(
        grid.width,
        2.0 * wide.width + CONTENT_GAP,
        "cells are equal"
    );
    assert_eq!(grid.height, tall.height);
    assert_eq!(grid.cells[0].y, (tall.height - wide.height) / 2.0);
    assert_eq!(
        grid.cells[1].x,
        wide.width + CONTENT_GAP + (wide.width - tall.width) / 2.0
    );
    assert_eq!(grid.cells[1].y, 0.0);
}

#[test]
fn content_that_cannot_report_a_size_keeps_the_placeholder_footprint() {
    let size = fitted(
        &json!({ "kind": "image", "title": "pending" }),
        &PLACEHOLDER,
        SizingMode::Content,
    );
    let grid = Grid::new(vec![size]).expect("grid");
    assert_eq!(grid.width, PLACEHOLDER.width);
    assert_eq!(grid.height, PLACEHOLDER.height);
}

#[test]
fn a_derivation_publishes_at_least_one_output() {
    assert!(Grid::new(Vec::new()).is_err());
}
