use super::*;

fn document(coverage_revision: &str) -> Value {
    json!({
        "kind": "file",
        "sourceRevision": "r1",
        "documentCoverage": {
            "sourceRevision": coverage_revision,
            "resources": [
                { "id": "a", "path": "/assets/a.png" },
                { "id": "b", "path": "/assets/b.png" }
            ],
            "occurrences": [
                { "location": { "kind": "pdf", "page": 1 }, "resourceId": "a" },
                { "location": { "kind": "pdf", "page": 2 }, "resourceId": "a" },
                { "location": { "kind": "slides", "slide": 3 }, "resourceId": "b" },
                { "location": { "kind": "pdf", "page": 4 }, "resourceId": null }
            ]
        }
    })
}

#[test]
fn reads_a_repeated_picture_once_from_its_first_appearance() {
    assert_eq!(
        sources(&document("r1")),
        [
            RecognitionSource {
                path: "/assets/a.png".into(),
                kind: "page",
                location: json!({ "kind": "pdf", "page": 1 }),
            },
            RecognitionSource {
                path: "/assets/b.png".into(),
                kind: "image_description",
                location: json!({ "kind": "slides", "slide": 3 }),
            },
        ]
    );
}

#[test]
fn a_stale_coverage_record_owes_nothing() {
    assert!(sources(&document("r0")).is_empty());
}

#[test]
fn images_are_read_unless_they_are_vectors_or_depth_fields() {
    let image = |path: &str| json!({ "kind": "image", "assetPath": path });
    assert_eq!(sources(&image("/assets/shot.PNG")).len(), 1);
    assert!(sources(&image("/assets/logo.svg")).is_empty());
    let mut depth = image("/assets/depth.png");
    depth["imageDepth"] = json!({});
    assert!(sources(&depth).is_empty());
}
