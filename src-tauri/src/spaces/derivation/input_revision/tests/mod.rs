use serde_json::json;

use super::{resolve, RevisionBasis};

#[test]
fn rendered_revision_tracks_image_edits_but_not_annotations() {
    let mut image = json!({"sourceRevision": "sha256:asset", "title": "First", "imageEdit": null});
    let original = resolve("image", &image, RevisionBasis::Rendered).unwrap();
    image["title"] = json!("Renamed");
    image["imageComments"] = json!([{"body": "Comment"}]);
    assert_eq!(
        resolve("image", &image, RevisionBasis::Rendered).unwrap(),
        original
    );

    image["imageEdit"] = json!({
        "version": 6,
        "crop": {"x": 0.0, "y": 0.0, "width": 1.0, "height": 1.0},
        "rotation": 90.0,
        "flipHorizontal": false,
        "flipVertical": false,
        "adjustments": {
            "exposure": 0.0, "contrast": 0.0, "highlights": 0.0,
            "shadows": 0.0, "whites": 0.0, "blacks": 0.0,
            "saturation": 0.0, "warmth": 0.0, "sharpness": 0.0,
            "vignette": 0.0, "vibrance": 0.0, "tint": 0.0,
            "clarity": 0.0, "grain": 0.0, "softLight": 0.0, "glow": 0.0
        },
    });
    assert_ne!(
        resolve("image", &image, RevisionBasis::Rendered).unwrap().0,
        original.0
    );
    assert_eq!(
        resolve("image", &image, RevisionBasis::Asset).unwrap().0,
        "sha256:asset"
    );
}
