use std::collections::HashSet;

use serde_json::json;

use super::{layouts, structure, valid_layouts, valid_structure};
use crate::spaces::structure::planner::{bounds::root_bounds, plan, tree_policy::Index};
use crate::spaces::structure::repository::SpaceStructureSnapshot;

#[test]
fn auto_layout_preserves_world_spacing_under_a_rotated_scaled_parent() {
    let mut snapshot = SpaceStructureSnapshot {
        structure: valid_structure(),
        layouts: valid_layouts(),
        relation_styles: Vec::new(),
    };
    snapshot.structure.extend(structure(json!([
        {"kind":"content","id":"c2","parentId":"g","order":1,"locked":false,"nodeId":"n2"}
    ])));
    let mut values = serde_json::to_value(&snapshot.layouts).unwrap();
    values[1]["transform"] = json!({"x":5,"y":10,"rotation":30,"scale":2});
    snapshot.layouts = layouts(values);
    snapshot.layouts.extend(layouts(json!([
        {"kind":"content","structureNodeId":"c2","transform":{"x":420,"y":500,"rotation":0,"scale":1},"size":{"width":300,"height":200},"sizingMode":"frame"}
    ])));
    snapshot.layouts.reverse();
    let before_structure = serde_json::to_value(&snapshot.structure).unwrap();
    let command = serde_json::from_value(json!({
        "kind":"structure.auto-layout","rootIds":["c2","c"],"flow":"row","gap":24
    }))
    .unwrap();
    plan(&command, &mut snapshot).unwrap();
    let index = Index::new(&snapshot.structure, &snapshot.layouts);
    let first = root_bounds(&snapshot.structure, &snapshot.layouts, &index, "c").unwrap();
    let second = root_bounds(&snapshot.structure, &snapshot.layouts, &index, "c2").unwrap();
    assert!((second.left - first.right - 24.0).abs() < 0.000001);
    assert!((second.top - first.top).abs() < 0.000001);
    assert_eq!(
        serde_json::to_value(&snapshot.structure).unwrap(),
        before_structure
    );
}

#[test]
fn removing_empty_groups_cascades_but_preserves_sections_and_nonempty_siblings() {
    let mut nodes = structure(json!([
        {"kind":"section","id":"s","parentId":null,"order":0,"locked":false,"title":"Area"},
        {"kind":"group","id":"a","parentId":"s","order":0,"locked":false,"title":"A"},
        {"kind":"group","id":"b","parentId":"a","order":0,"locked":false,"title":"B"},
        {"kind":"group","id":"c","parentId":"b","order":0,"locked":false,"title":"C"},
        {"kind":"group","id":"g","parentId":"s","order":1,"locked":false,"title":"Keep"},
        {"kind":"content","id":"k","parentId":"g","order":0,"locked":false,"nodeId":"n"}
    ]));
    let mut frames = layouts(json!([
        {"kind":"group","structureNodeId":"a","transform":{"x":0,"y":0,"rotation":0,"scale":1}},
        {"kind":"group","structureNodeId":"b","transform":{"x":0,"y":0,"rotation":0,"scale":1}},
        {"kind":"group","structureNodeId":"c","transform":{"x":0,"y":0,"rotation":0,"scale":1}},
        {"kind":"group","structureNodeId":"g","transform":{"x":0,"y":0,"rotation":0,"scale":1}}
    ]));
    let removed = crate::spaces::structure::planner::tree_policy::remove_empty_groups(
        &mut nodes,
        &mut frames,
    );
    assert_eq!(
        removed,
        HashSet::from(["a".to_owned(), "b".to_owned(), "c".to_owned()])
    );
    assert_eq!(
        nodes.iter().map(|node| node.id()).collect::<Vec<_>>(),
        ["s", "g", "k"]
    );
    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0].structure_node_id(), "g");
}

#[test]
fn batch_heights_find_shuffled_layouts_without_changing_other_geometry() {
    let count = 1000;
    let mut snapshot = SpaceStructureSnapshot {
        structure: structure((0..count).map(|i| json!({
            "kind":"content","id":format!("c{i}"),"nodeId":format!("n{i}"),"parentId":null,"order":i,"locked":false
        })).collect()),
        layouts: layouts((0..count).rev().map(|i| json!({
            "kind":"content","structureNodeId":format!("c{i}"),"transform":{"x":i,"y":10,"rotation":0,"scale":1},"size":{"width":300,"height":200},"sizingMode":"frame"
        })).collect()),
        relation_styles: Vec::new(),
    };
    let command = serde_json::from_value(json!({
        "kind":"content.update-heights","updates":(0..count).map(|i| json!({
            "structureNodeId":format!("c{i}"),"height":240+i,"measuredWidth":300,"sizingMode":"frame","typographyScale":1
        })).collect::<Vec<_>>()
    })).unwrap();
    plan(&command, &mut snapshot).unwrap();
    let values = serde_json::to_value(&snapshot.layouts).unwrap();
    for (position, layout) in values.as_array().unwrap().iter().enumerate() {
        let id = count - position - 1;
        assert_eq!(
            layout["size"],
            json!({"width":300.0,"height":(240+id) as f64})
        );
        assert_eq!(
            layout["transform"],
            json!({"x":id as f64,"y":10.0,"rotation":0.0,"scale":1.0})
        );
    }
}
