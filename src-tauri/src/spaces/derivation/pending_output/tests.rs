use serde_json::json;

use super::{fitted, Grid, PendingOutput, PLACEHOLDER};
use crate::spaces::structure::CONTENT_GAP;

fn layer(width: u32, height: u32) -> PendingOutput {
    PendingOutput {
        node_id: format!("layer-{width}x{height}"),
        kind: "image",
        data: json!({
            "kind": "image", "title": "layer",
            "intrinsicWidth": width, "intrinsicHeight": height
        }),
    }
}

#[test]
fn a_set_of_outputs_is_laid_out_as_a_grid_rather_than_one_row() {
    let outputs = (0..9).map(|_| layer(400, 400)).collect::<Vec<_>>();
    let cell = fitted(&outputs[0].data);
    let grid = Grid::new(&outputs).expect("grid");
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
    let outputs = (0..8).map(|_| layer(400, 400)).collect::<Vec<_>>();
    let cell = fitted(&outputs[0].data);
    let grid = Grid::new(&outputs).expect("grid");
    assert_eq!(grid.height, 3.0 * cell.height + 2.0 * CONTENT_GAP);
    assert_eq!(grid.cells[7].y, 2.0 * (cell.height + CONTENT_GAP));
    assert_eq!(grid.cells[7].x, cell.width + CONTENT_GAP);
}

#[test]
fn each_output_keeps_its_own_size_and_is_centred_in_its_cell() {
    let outputs = vec![layer(1600, 400), layer(400, 1600)];
    let wide = fitted(&outputs[0].data);
    let tall = fitted(&outputs[1].data);
    assert!(
        wide.width > tall.width && tall.height > wide.height,
        "the two layers are sized by their own aspect"
    );
    let grid = Grid::new(&outputs).expect("grid");
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
fn an_output_that_cannot_report_a_size_keeps_the_placeholder_footprint() {
    let output = PendingOutput {
        node_id: "pending".into(),
        kind: "image",
        data: json!({ "kind": "image", "title": "pending" }),
    };
    let grid = Grid::new(std::slice::from_ref(&output)).expect("grid");
    assert_eq!(grid.width, PLACEHOLDER.width);
    assert_eq!(grid.height, PLACEHOLDER.height);
}

#[test]
fn a_derivation_reserves_at_least_one_output() {
    assert!(Grid::new(&[]).is_err());
}
