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

fn resource(id: &str) -> Picture {
    Picture::Resource {
        id: id.into(),
        path: format!("/assets/{id}.png"),
    }
}

#[test]
fn reads_a_repeated_picture_once_from_its_first_appearance() {
    assert_eq!(
        sources(&document("r1")),
        [
            RecognitionSource {
                picture: resource("a"),
                kind: "page",
                location: json!({ "kind": "pdf", "page": 1 }),
            },
            RecognitionSource {
                picture: resource("b"),
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
fn a_picture_node_is_read_as_it_is_shown_unless_it_is_a_depth_field() {
    let shown = |kind| RecognitionSource {
        picture: Picture::Shown,
        kind,
        location: json!({ "kind": "node" }),
    };
    assert_eq!(
        sources(&json!({ "kind": "image" })),
        [shown("image_description")]
    );
    assert_eq!(sources(&json!({ "kind": "video" })), [shown("video_cover")]);
    assert!(sources(&json!({ "kind": "image", "imageDepth": {} })).is_empty());
    assert!(sources(&json!({ "kind": "audio" })).is_empty());
}
