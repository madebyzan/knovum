use super::*;
use crate::spaces::structure::{Point, Size};

fn block<'a>(
    sources: &'a [String],
    placed: &'a [String],
    siblings: usize,
    width: f64,
    height: f64,
) -> DerivedBlock<'a> {
    DerivedBlock {
        source_node_ids: sources,
        placed_node_ids: placed,
        siblings,
        width,
        height,
    }
}

#[test]
fn unanchored_content_is_placed_beside_sections() {
    let structure = vec![StructureNode::Section {
        id: "section".into(),
        parent_id: None,
        order: 0,
        locked: false,
        title: "Section".into(),
    }];
    let layouts = vec![StructureLayout::Section {
        structure_node_id: "section".into(),
        position: Point { x: 0.0, y: 0.0 },
        size: Size {
            width: 400.0,
            height: 300.0,
        },
        collapsed: false,
        appearance: None,
        auto_layout: false,
    }];
    assert_eq!(
        place_new_content(&structure, &layouts, 280.0, 350.0).unwrap(),
        (448.0, 0.0)
    );
    assert_eq!(
        place_new_content(&[], &[], 280.0, 350.0).unwrap(),
        (0.0, 0.0)
    );
}

#[test]
fn content_being_placed_does_not_block_its_own_block() {
    // A derivation reserved one output beside its source before it knew the set would hold four,
    // so the set must be able to claim the area that output already sits in.
    let structure = vec![
        content("source", "source-node", 0),
        content("reserved", "reserved-node", 1),
    ];
    let layouts = vec![
        content_layout("source", 0.0, 0.0, 400.0, 300.0),
        content_layout("reserved", 448.0, 0.0, 320.0, 220.0),
    ];
    let sources = ["source-node".to_owned()];
    let placed = ["reserved-node".to_owned()];
    assert_eq!(
        place_derived_content(
            &structure,
            &layouts,
            &block(&sources, &placed, 0, 700.0, 500.0)
        )
        .unwrap(),
        (448.0, 0.0),
        "the set starts where its output sits"
    );
    assert_eq!(
        place_derived_content(&structure, &layouts, &block(&sources, &[], 0, 700.0, 500.0))
            .unwrap(),
        (448.0, 548.0),
        "an output that is not part of the set is followed instead of overlapped"
    );
}

#[test]
fn a_field_fills_its_row_before_starting_the_next() {
    // The source already has one derived output sitting in the first cell of its field.
    let structure = vec![
        content("source", "source-node", 0),
        content("output", "output-node", 1),
    ];
    let layouts = vec![
        content_layout("source", 0.0, 0.0, 400.0, 300.0),
        content_layout("output", 448.0, 0.0, 320.0, 220.0),
    ];
    let sources = ["source-node".to_owned()];
    assert_eq!(
        place_derived_content(&structure, &layouts, &block(&sources, &[], 1, 320.0, 220.0))
            .unwrap(),
        (816.0, 0.0),
        "a second derivation widens the field into its next column"
    );
    assert_eq!(
        place_derived_content(&structure, &layouts, &block(&sources, &[], 0, 320.0, 220.0))
            .unwrap(),
        (448.0, 268.0),
        "a single-column field continues downward"
    );
    assert_eq!(field_columns(0), 1);
    assert_eq!(field_columns(3), 2);
    assert_eq!(field_columns(8), 3);
}

fn content(id: &str, node_id: &str, order: u32) -> StructureNode {
    StructureNode::Content {
        id: id.into(),
        parent_id: None,
        order,
        locked: false,
        node_id: node_id.into(),
    }
}

fn content_layout(id: &str, x: f64, y: f64, width: f64, height: f64) -> StructureLayout {
    StructureLayout::Content {
        structure_node_id: id.into(),
        transform: crate::spaces::structure::LocalTransform {
            x,
            y,
            rotation: 0.0,
            scale: 1.0,
        },
        size: Size { width, height },
        sizing_mode: crate::spaces::structure::SizingMode::Content,
        appearance: None,
        file_subpath: None,
    }
}
