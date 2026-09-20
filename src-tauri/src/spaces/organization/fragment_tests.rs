use crate::spaces::canvas_state::{self, contract::SpaceEditorDocument};
use super::{fragment_contract::{EditorFragment, FragmentPlacement}, fragment_plan};

fn fragment() -> EditorFragment {
    serde_json::from_value(serde_json::json!({
        "structure":{"version":1,"items":[
            {"kind":"group","id":"group","title":"Selection","parentId":null,"order":0,"locked":false},
            {"kind":"content","id":"node","nodeId":"node","parentId":"group","order":0,"locked":false}
        ]},
        "layout":{"version":1,"items":[
            {"kind":"group","itemId":"group","transform":{"x":100,"y":200,"rotation":30,"scale":2}},
            {"kind":"content","itemId":"node","transform":{"x":25,"y":10,"rotation":0,"scale":1},"size":{"width":120,"height":80},"sizeAuthority":"frame"}
        ]},
        "nodes":[{"id":"node","kind":"note","data":{"kind":"note","title":"A","tiptapContent":{"type":"doc","content":[]},"plainText":""},"sourceChunks":[]}],
        "relations":[],"objectTypes":[],"relationStyles":[]
    })).expect("fragment")
}
fn placement() -> FragmentPlacement {
    serde_json::from_value(serde_json::json!({"parentId":null,"delta":{"x":24,"y":24}})).expect("placement")
}
#[test]
fn preserves_nested_local_geometry_and_moves_only_fragment_roots() {
    let source = fragment();
    let result = fragment_plan::plan(&SpaceEditorDocument::empty(), &source, &placement()).expect("plan");
    assert_eq!(result.after.layout.items[1], source.layout.items[1]);
    let world = canvas_state::geometry::project(&result.after).expect("geometry");
    assert!((world.world["group"].x - 124.0).abs() < 1e-9);
    assert_eq!(result.selection, vec!["group"]);
}
#[test]
fn rejects_missing_content_without_partially_mutating_the_destination() {
    let mut source = fragment(); source.nodes.clear();
    let target = SpaceEditorDocument::empty();
    assert!(fragment_plan::plan(&target,&source,&placement()).is_err());
    assert!(target.structure.items.is_empty());
}
#[test]
fn rejects_duplicate_structure_identity_in_the_target() {
    let source = fragment();
    let target = fragment_plan::plan(&SpaceEditorDocument::empty(),&source,&placement()).expect("first").after;
    assert!(fragment_plan::plan(&target,&source,&placement()).is_err());
}
