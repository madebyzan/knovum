use serde_json::json;

use super::{fitted_size, Size, SizingMode};

#[test]
fn content_dimensions_follow_the_displayed_media_ratio() {
    let current = Size {
        width: 256.0,
        height: 156.0,
    };
    let image = json!({"kind":"image","intrinsicWidth":1920,"intrinsicHeight":1080});
    let landscape = fitted_size(&image, &current, SizingMode::Content).unwrap();
    assert_eq!((landscape.width, landscape.height), (280.0, 158.0));

    let edited = json!({
        "kind":"image","intrinsicWidth":1920,"intrinsicHeight":1080,
        "imageEdit":{"crop":{"width":0.5,"height":1.0},"rotation":90.0}
    });
    let rotated = fitted_size(&edited, &current, SizingMode::Content).unwrap();
    assert_eq!((rotated.width, rotated.height), (280.0, 249.0));

    let video = json!({"kind":"video","intrinsicWidth":1080,"intrinsicHeight":1920});
    let portrait = fitted_size(&video, &current, SizingMode::Content).unwrap();
    assert_eq!((portrait.width, portrait.height), (171.0, 304.0));

    let document = json!({"kind":"file","intrinsicWidth":210,"intrinsicHeight":297});
    let page = fitted_size(&document, &current, SizingMode::Content).unwrap();
    assert_eq!((page.width, page.height), (192.0, 320.0));
}

#[test]
fn framed_media_keeps_its_area_when_the_ratio_changes() {
    let current = Size {
        width: 300.0,
        height: 200.0,
    };
    let video = json!({"kind":"video","intrinsicWidth":1080,"intrinsicHeight":1920});
    let fitted = fitted_size(&video, &current, SizingMode::Frame).unwrap();
    assert!((fitted.width * fitted.height - 60_000.0).abs() < 0.01);
    assert!((fitted.width / fitted.height - 0.5625).abs() < 0.0001);
}
