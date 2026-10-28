use serde_json::{json, Value};

use super::{layouts, structure};
use crate::spaces::structure::planner::{
    bounds::root_bounds, plan, settle_auto_layout_sections, tree_policy::Index,
};
use crate::spaces::structure::repository::SpaceStructureSnapshot;
use crate::spaces::structure::StructureLayout;

/// Four 100×80 cards in a Section, deliberately out of reading order in their sibling order.
fn frame(width: f64, auto_layout: bool) -> SpaceStructureSnapshot {
    let cards = [
        ("d", 400, 300),
        ("a", 40, 60),
        ("c", 60, 220),
        ("b", 300, 70),
    ];
    let mut nodes = vec![
        json!({"kind":"section","id":"s","parentId":null,"order":0,"locked":false,"title":"Frame"}),
    ];
    let mut frames = vec![json!({
        "kind":"section","structureNodeId":"s","position":{"x":1000,"y":2000},
        "size":{"width":width,"height":400},"collapsed":false,"autoLayout":auto_layout
    })];
    for (order, (id, x, y)) in cards.iter().enumerate() {
        nodes.push(json!({"kind":"content","id":id,"parentId":"s","order":order,"locked":false,"nodeId":format!("n{id}")}));
        frames.push(json!({
            "kind":"content","structureNodeId":id,"transform":{"x":x,"y":y,"rotation":0,"scale":1},
            "size":{"width":100,"height":80},"sizingMode":"frame"
        }));
    }
    SpaceStructureSnapshot {
        structure: structure(Value::Array(nodes)),
        layouts: layouts(Value::Array(frames)),
        relation_styles: Vec::new(),
    }
}

fn origin(snapshot: &SpaceStructureSnapshot, id: &str) -> (f64, f64) {
    let index = Index::new(&snapshot.structure, &snapshot.layouts);
    let bounds = root_bounds(&snapshot.structure, &snapshot.layouts, &index, id).unwrap();
    (bounds.left, bounds.top)
}

fn section_size(snapshot: &SpaceStructureSnapshot) -> (f64, f64) {
    match snapshot
        .layouts
        .iter()
        .find(|layout| layout.structure_node_id() == "s")
    {
        Some(StructureLayout::Section { size, .. }) => (size.width, size.height),
        _ => panic!("Section layout missing"),
    }
}

fn execute(snapshot: &mut SpaceStructureSnapshot, command: Value) {
    plan(&serde_json::from_value(command).unwrap(), snapshot).unwrap();
    settle_auto_layout_sections(snapshot).unwrap();
}

#[test]
fn auto_layout_section_packs_children_in_reading_order_and_hugs_their_rows() {
    // 32 + 3 × 100 + 2 × 32 + 32 = 428: three columns fit, the fourth card wraps.
    let mut snapshot = frame(430.0, true);
    assert!(settle_auto_layout_sections(&mut snapshot).unwrap());
    assert_eq!(origin(&snapshot, "a"), (1032.0, 2056.0));
    assert_eq!(origin(&snapshot, "b"), (1164.0, 2056.0));
    assert_eq!(origin(&snapshot, "c"), (1296.0, 2056.0));
    assert_eq!(origin(&snapshot, "d"), (1032.0, 2168.0));
    assert_eq!(
        section_size(&snapshot),
        (430.0, 56.0 + 80.0 + 32.0 + 80.0 + 32.0)
    );
}

#[test]
fn settling_a_settled_section_changes_nothing() {
    let mut snapshot = frame(430.0, true);
    settle_auto_layout_sections(&mut snapshot).unwrap();
    let settled = snapshot.layouts.clone();
    settle_auto_layout_sections(&mut snapshot).unwrap();
    assert_eq!(snapshot.layouts, settled);
}

#[test]
fn a_child_dropped_between_two_others_is_packed_between_them() {
    let mut snapshot = frame(430.0, true);
    settle_auto_layout_sections(&mut snapshot).unwrap();
    // d sits alone on the second row; drop it onto the first, just right of a.
    execute(
        &mut snapshot,
        json!({"kind":"structure.move","rootIds":["d"],"delta":{"x":60,"y":-112}}),
    );
    assert_eq!(origin(&snapshot, "a"), (1032.0, 2056.0));
    assert_eq!(origin(&snapshot, "d"), (1164.0, 2056.0));
    assert_eq!(origin(&snapshot, "b"), (1296.0, 2056.0));
    assert_eq!(origin(&snapshot, "c"), (1032.0, 2168.0));
}

#[test]
fn width_decides_the_columns_and_a_child_wider_than_the_section_widens_it() {
    let mut snapshot = frame(430.0, true);
    execute(
        &mut snapshot,
        json!({"kind":"section.resize-frame","structureNodeId":"s","frame":{"x":1000,"y":2000,"width":1000,"height":400}}),
    );
    assert_eq!(origin(&snapshot, "d"), (1428.0, 2056.0));
    assert_eq!(section_size(&snapshot), (1000.0, 56.0 + 80.0 + 32.0));
    let mut values = serde_json::to_value(&snapshot.layouts).unwrap();
    values[2]["size"]["width"] = json!(400);
    snapshot.layouts = layouts(values);
    execute(
        &mut snapshot,
        json!({"kind":"section.resize-frame","structureNodeId":"s","frame":{"x":1000,"y":2000,"width":240,"height":400}}),
    );
    assert_eq!(section_size(&snapshot).0, 400.0 + 64.0);
    assert_eq!(origin(&snapshot, "b"), (1032.0, 2168.0));
}

#[test]
fn turning_auto_layout_on_and_off_is_a_section_layout_update() {
    let mut snapshot = frame(430.0, false);
    assert!(!settle_auto_layout_sections(&mut snapshot).unwrap());
    assert_eq!(origin(&snapshot, "d"), (1400.0, 2300.0));
    execute(
        &mut snapshot,
        json!({"kind":"section.update-layout","structureNodeId":"s","autoLayout":true,
               "appearance":{"kind":"set","value":{"fill":"blue"}}}),
    );
    assert_eq!(origin(&snapshot, "d"), (1032.0, 2168.0));
    execute(
        &mut snapshot,
        json!({"kind":"section.update-layout","structureNodeId":"s","autoLayout":false}),
    );
    let section = serde_json::to_value(&snapshot.layouts[0]).unwrap();
    assert_eq!(section["appearance"], json!({"fill":"blue"}));
    assert!(section.get("autoLayout").is_none());
    let content = serde_json::from_value(
        json!({"kind":"section.update-layout","structureNodeId":"a","autoLayout":true}),
    )
    .unwrap();
    assert!(plan(&content, &mut snapshot).is_err());
}

#[test]
fn tidy_up_packs_the_selection_into_the_squarest_table_from_its_top_left() {
    let mut snapshot = frame(430.0, false);
    execute(
        &mut snapshot,
        json!({"kind":"structure.auto-layout","rootIds":["a","b","c","d"],"flow":"grid","gap":32}),
    );
    let start = (1040.0, 2060.0);
    assert_eq!(origin(&snapshot, "a"), start);
    assert_eq!(origin(&snapshot, "b"), (start.0 + 132.0, start.1));
    assert_eq!(origin(&snapshot, "c"), (start.0, start.1 + 112.0));
    assert_eq!(origin(&snapshot, "d"), (start.0 + 132.0, start.1 + 112.0));
}

#[test]
fn row_and_column_follow_reading_order_rather_than_sibling_order() {
    let mut snapshot = frame(430.0, false);
    execute(
        &mut snapshot,
        json!({"kind":"structure.auto-layout","rootIds":["a","b","c","d"],"flow":"column","gap":10}),
    );
    let lefts_and_tops = ["a", "b", "c", "d"].map(|id| origin(&snapshot, id));
    assert_eq!(
        lefts_and_tops,
        [
            (1040.0, 2060.0),
            (1040.0, 2150.0),
            (1040.0, 2240.0),
            (1040.0, 2330.0)
        ]
    );
}

#[test]
fn stored_sections_without_the_field_are_freely_arranged() {
    let stored = json!({"kind":"section","structureNodeId":"s","position":{"x":0.0,"y":0.0},"size":{"width":640.0,"height":400.0},"collapsed":false});
    let layout: StructureLayout = serde_json::from_value(stored.clone()).unwrap();
    assert!(matches!(
        layout,
        StructureLayout::Section {
            auto_layout: false,
            ..
        }
    ));
    assert_eq!(serde_json::to_value(&layout).unwrap(), stored);
}

#[test]
fn an_auto_layout_section_keeps_arranging_its_own_children() {
    let mut snapshot = frame(430.0, true);
    for command in [
        json!({"kind":"structure.align","rootIds":["a","b"],"edge":"left"}),
        json!({"kind":"structure.distribute","rootIds":["a","b","c"],"axis":"vertical"}),
        json!({"kind":"structure.auto-layout","rootIds":["a","b"],"flow":"grid","gap":32}),
    ] {
        let command = serde_json::from_value(command).unwrap();
        assert!(plan(&command, &mut snapshot).is_err());
    }
}
