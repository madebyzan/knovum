use std::path::Path;

use serde_json::{json, Value};

use super::targets::{self, Export, Managed};
use crate::documents::generation::DocumentFormat;

fn offered(payload: &Value) -> Vec<String> {
    targets::of(payload).unwrap()
}

#[test]
fn an_untouched_picture_keeps_its_own_format_and_can_be_encoded_again() {
    let payload = json!({"kind":"image","fileName":"cover.webp","assetPath":"/owned/cover.webp"});
    assert_eq!(offered(&payload), ["webp", "png", "jpg"]);
    assert_eq!(
        targets::requested(&payload, Path::new("/out/cover.webp")).unwrap(),
        Export::Managed(Managed::Original),
        "its own format is the bytes it already has, not a re-encode"
    );
    assert!(matches!(
        targets::requested(&payload, Path::new("/out/cover.JPEG")).unwrap(),
        Export::Managed(Managed::Image(_)),
    ));
}

#[test]
fn an_edited_picture_is_only_offered_as_a_new_encoding() {
    let payload = json!({
        "kind":"image", "fileName":"cover.png", "assetPath":"/owned/cover.png",
        "imageEdit":{"crop":{"x":0.0,"y":0.0,"width":1.0,"height":1.0}},
    });
    assert_eq!(offered(&payload), ["png", "jpg"]);
    assert!(matches!(
        targets::requested(&payload, Path::new("/out/cover.png")).unwrap(),
        Export::Managed(Managed::Image(_)),
    ));
}

#[test]
fn a_depth_field_is_measurement_data_rather_than_a_picture() {
    let payload = json!({
        "kind":"image", "fileName":"depth.png", "assetPath":"/owned/depth.png",
        "imageDepth":{"encoding":"relative-inverse-depth-u16-v1","modelSpecId":"spec"},
    });
    assert_eq!(offered(&payload), ["png"]);
    assert_eq!(
        targets::requested(&payload, Path::new("/out/depth.png")).unwrap(),
        Export::Managed(Managed::Original),
    );
    assert!(targets::requested(&payload, Path::new("/out/depth.jpg")).is_err());
}

#[test]
fn recordings_offer_their_own_container_and_the_qualified_encodings() {
    assert_eq!(
        offered(&json!({"kind":"audio","fileName":"take.m4a","assetPath":"/owned/take.m4a"})),
        ["m4a", "wav", "mp3", "flac"]
    );
    assert_eq!(
        offered(&json!({"kind":"video","fileName":"clip.mkv","assetPath":"/owned/clip.mkv"})),
        ["mkv", "mp4", "mov"]
    );
}

#[test]
fn documents_reach_the_formats_the_engine_serializes() {
    assert_eq!(
        offered(&json!({"kind":"note","title":"Notes"})),
        ["txt", "html", "docx", "pdf", "pptx"]
    );
    assert_eq!(
        targets::requested(
            &json!({"kind":"markdown","title":"D","text":"x"}),
            Path::new("/o.pdf")
        )
        .unwrap(),
        Export::Document(DocumentFormat::Pdf)
    );
}

/// Original-layout PDF of an Office file is outside the product boundary, and a
/// deck is only derived from content that carries one.
#[test]
fn office_files_keep_their_own_bytes_and_offer_only_semantic_content() {
    let office = |extension: &str| {
        json!({"kind":"file","title":"Report","fileName":format!("report.{extension}"),
               "assetPath":format!("/owned/report.{extension}"),"sourceRevision":"sha256:current"})
    };
    for extension in ["doc", "docx", "xls", "xlsx", "pptx"] {
        let payload = office(extension);
        assert!(!offered(&payload).contains(&"pdf".to_owned()));
        assert!(!offered(&payload).contains(&"pptx".to_owned()) || extension == "pptx");
        assert_eq!(
            targets::requested(&payload, Path::new(&format!("/out/report.{extension}"))).unwrap(),
            Export::Managed(Managed::Original),
        );
    }
    assert_eq!(
        offered(&office("pdf")),
        ["pdf", "docx", "txt", "html", "pptx"]
    );
    assert_eq!(
        offered(&office("ppt")),
        ["ppt", "docx", "txt", "html", "pptx"]
    );
}

#[test]
fn a_file_with_no_conversion_route_is_still_saved_as_itself() {
    let payload = json!({"kind":"file","title":"Archive","fileName":"bundle.zip",
                         "assetPath":"/owned/bundle.zip","sourceRevision":"sha256:current"});
    assert_eq!(offered(&payload), ["zip"]);
    assert!(targets::requested(&payload, Path::new("/out/bundle.txt"))
        .unwrap_err()
        .contains("cannot be exported as .txt"));
}

#[test]
fn unsettled_resources_report_why_they_have_no_file_yet() {
    assert!(targets::of(&json!({
        "kind":"image", "fileName":"cover.png", "assetPath":"/owned/cover.png",
        "processingState":"processing"
    }))
    .unwrap_err()
    .contains("Wait for this resource"));
    assert!(targets::of(
        &json!({"kind":"markdown","title":"D","text":"","derivationState":"pending"})
    )
    .unwrap_err()
    .contains("Wait for this resource"));
}

/// A website summary or an object is serialized by the Renderer from content it
/// already holds, so the engine truthfully offers nothing rather than failing.
#[test]
fn renderer_owned_content_is_offered_no_engine_format() {
    assert!(targets::of(&json!({"kind":"website","title":"Page"}))
        .unwrap()
        .is_empty());
}

#[test]
fn a_destination_without_an_extension_names_no_format() {
    assert!(targets::requested(
        &json!({"kind":"note","title":"Notes"}),
        Path::new("/out/notes")
    )
    .unwrap_err()
    .contains("needs a file extension"));
}
