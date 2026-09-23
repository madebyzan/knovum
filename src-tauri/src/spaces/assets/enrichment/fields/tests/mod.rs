use super::apply_intrinsic_geometry;
use serde_json::{json, Map};

#[test]
fn removes_visual_geometry_from_audio_projection() {
    let mut object = Map::from_iter([
        ("intrinsicWidth".into(), json!(640)),
        ("intrinsicHeight".into(), json!(480)),
    ]);
    apply_intrinsic_geometry(&mut object, "audio", Some(640), Some(480)).unwrap();
    assert!(!object.contains_key("intrinsicWidth"));
    assert!(!object.contains_key("intrinsicHeight"));
}
