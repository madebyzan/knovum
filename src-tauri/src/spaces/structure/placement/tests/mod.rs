use super::*;
use crate::spaces::structure::{Point, Size};

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
    }];
    assert_eq!(
        place_new_content(&structure, &layouts, &[], 280.0, 350.0).unwrap(),
        (448.0, 0.0)
    );
    assert_eq!(
        place_new_content(&[], &[], &[], 280.0, 350.0).unwrap(),
        (0.0, 0.0)
    );
}
