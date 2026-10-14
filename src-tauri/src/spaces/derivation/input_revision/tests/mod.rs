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
        "version": 1,
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

#[test]
fn rendered_revision_tracks_video_edits_at_current_version() {
    let project = json!({
        "version": crate::spaces::derivation::video_edit::PROJECT_VERSION,
        "sourceRevision": "sha256:asset",
        "assets": [{"nodeId": "source", "sourceRevision": "sha256:asset", "kind": "video", "durationMs": 8000, "width": 1080, "height": 1920}],
        "output": {"width": 1080, "height": 1920, "frameRate": 30, "background": "#000000"},
        "videoTrack": {"clips": [{"id": "d51a4384-70ab-4b44-90cc-3bfcf0f5341f", "assetId": "source", "sourceInMs": 0, "sourceOutMs": 8000, "timelineStartMs": 0, "volume": 1.0, "fadeInMs": 0, "fadeOutMs": 0, "transform": {"x": 0.0, "y": 0.0, "scale": 1.0, "rotation": 0.0, "opacity": 1.0, "fit": "contain", "flipHorizontal": false, "flipVertical": false}}]},
        "transitions": [],
        "audioTrack": {"clips": []},
        "textTrack": {"items": []}
    });
    let video = json!({
        "sourceRevision": "sha256:asset",
        "title": "Video",
        "videoEditProject": project,
    });
    let result = resolve("video", &video, RevisionBasis::Rendered);
    assert!(result.is_ok());
    assert!(result.unwrap().0.starts_with("rendered1:"));

    let mut mismatched = video.clone();
    mismatched["sourceRevision"] = json!("sha256:other");
    assert_eq!(
        resolve("video", &mismatched, RevisionBasis::Rendered).unwrap_err(),
        "Video edit project does not match its asset"
    );
}
