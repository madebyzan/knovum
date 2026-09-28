use std::collections::HashSet;

mod transactions;

use serde_json::{json, Value};

use super::{validate_content_coverage, validate_space_snapshot, StructureLayout, StructureNode};

fn structure(values: Value) -> Vec<StructureNode> {
    serde_json::from_value(values).unwrap()
}

fn layouts(values: Value) -> Vec<StructureLayout> {
    serde_json::from_value(values).unwrap()
}

fn valid_structure() -> Vec<StructureNode> {
    structure(json!([
        {"kind":"section","id":"s","parentId":null,"order":0,"locked":false,"title":"Area"},
        {"kind":"group","id":"g","parentId":"s","order":0,"locked":false,"title":"Group"},
        {"kind":"content","id":"c","parentId":"g","order":0,"locked":false,"nodeId":"n"}
    ]))
}

fn valid_layouts() -> Vec<StructureLayout> {
    layouts(json!([
        {"kind":"section","structureNodeId":"s","position":{"x":0,"y":0},"size":{"width":640,"height":400},"collapsed":false},
        {"kind":"group","structureNodeId":"g","transform":{"x":5,"y":10,"rotation":0,"scale":1}},
        {"kind":"content","structureNodeId":"c","transform":{"x":20,"y":30,"rotation":0,"scale":1},"size":{"width":300,"height":200},"sizingMode":"frame"}
    ]))
}

#[test]
fn accepts_valid_tree_and_active_content_coverage() {
    let structure = valid_structure();
    assert!(validate_space_snapshot(&structure, &valid_layouts()).is_ok());
    assert!(validate_content_coverage(&structure, &HashSet::from(["n".to_owned()])).is_ok());
}

#[test]
fn rejects_invalid_parent_and_cycle() {
    let section_child = structure(json!([
        {"kind":"section","id":"a","parentId":"b","order":0,"locked":false,"title":"A"},
        {"kind":"group","id":"b","parentId":null,"order":0,"locked":false,"title":"B"}
    ]));
    assert!(validate_space_snapshot(&section_child, &[]).is_err());
    let cycle = structure(json!([
        {"kind":"group","id":"a","parentId":"b","order":0,"locked":false,"title":"A"},
        {"kind":"group","id":"b","parentId":"a","order":0,"locked":false,"title":"B"}
    ]));
    assert!(validate_space_snapshot(&cycle, &[]).is_err());
}

#[test]
fn rejects_duplicate_content_and_noncanonical_siblings() {
    let duplicate = structure(json!([
        {"kind":"content","id":"a","parentId":null,"order":0,"locked":false,"nodeId":"n"},
        {"kind":"content","id":"b","parentId":null,"order":1,"locked":false,"nodeId":"n"}
    ]));
    assert!(validate_space_snapshot(&duplicate, &[]).is_err());
    let gap = structure(json!([
        {"kind":"group","id":"a","parentId":null,"order":1,"locked":false,"title":"A"}
    ]));
    assert!(validate_space_snapshot(&gap, &[]).is_err());
}

#[test]
fn rejects_layout_mismatch_and_degenerate_scale() {
    let mut missing = valid_layouts();
    missing.pop();
    assert!(validate_space_snapshot(&valid_structure(), &missing).is_err());
    let invalid = layouts(json!([
        {"kind":"section","structureNodeId":"s","position":{"x":0,"y":0},"size":{"width":640,"height":400},"collapsed":false},
        {"kind":"group","structureNodeId":"g","transform":{"x":5,"y":10,"rotation":0,"scale":0}},
        {"kind":"content","structureNodeId":"c","transform":{"x":20,"y":30,"rotation":0,"scale":1},"size":{"width":300,"height":200},"sizingMode":"frame"}
    ]));
    assert!(validate_space_snapshot(&valid_structure(), &invalid).is_err());
}

#[test]
fn rejects_section_below_minimum_size() {
    let small = layouts(json!([
        {"kind":"section","structureNodeId":"s","position":{"x":0,"y":0},"size":{"width":239,"height":160},"collapsed":false},
        {"kind":"group","structureNodeId":"g","transform":{"x":5,"y":10,"rotation":0,"scale":1}},
        {"kind":"content","structureNodeId":"c","transform":{"x":20,"y":30,"rotation":0,"scale":1},"size":{"width":300,"height":200},"sizingMode":"frame"}
    ]));
    assert!(validate_space_snapshot(&valid_structure(), &small).is_err());
}

#[test]
fn rejects_unknown_contract_fields_and_missing_active_content() {
    let invalid = json!({"kind":"section","id":"s","parentId":null,"order":0,"locked":false,"title":"S","nodeId":"n"});
    assert!(serde_json::from_value::<StructureNode>(invalid).is_err());
    assert!(validate_content_coverage(&valid_structure(), &HashSet::new()).is_err());
}

#[test]
fn rejects_empty_group_and_utf16_id_over_limit() {
    let empty = structure(json!([
        {"kind":"group","id":"g","parentId":null,"order":0,"locked":false,"title":"Group"}
    ]));
    assert!(validate_space_snapshot(&empty, &[]).is_err());
    let too_long = "😀".repeat(65);
    let invalid = structure(json!([
        {"kind":"section","id":too_long,"parentId":null,"order":0,"locked":false,"title":"Area"}
    ]));
    assert!(validate_space_snapshot(&invalid, &[]).is_err());
}

#[test]
fn rejects_invalid_appearance_and_accumulated_rotation() {
    let invalid_appearance = layouts(json!([
        {"kind":"section","structureNodeId":"s","position":{"x":0,"y":0},"size":{"width":640,"height":400},"collapsed":false,"appearance":{"textColor":"red"}},
        {"kind":"group","structureNodeId":"g","transform":{"x":5,"y":10,"rotation":0,"scale":1}},
        {"kind":"content","structureNodeId":"c","transform":{"x":20,"y":30,"rotation":0,"scale":1},"size":{"width":300,"height":200},"sizingMode":"frame"}
    ]));
    assert!(validate_space_snapshot(&valid_structure(), &invalid_appearance).is_err());
    let overflow = layouts(json!([
        {"kind":"section","structureNodeId":"s","position":{"x":0,"y":0},"size":{"width":640,"height":400},"collapsed":false},
        {"kind":"group","structureNodeId":"g","transform":{"x":5,"y":10,"rotation":1000000000,"scale":1}},
        {"kind":"content","structureNodeId":"c","transform":{"x":20,"y":30,"rotation":1,"scale":1},"size":{"width":300,"height":200},"sizingMode":"frame"}
    ]));
    assert!(validate_space_snapshot(&valid_structure(), &overflow).is_err());
}
