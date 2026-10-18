use std::path::Path;

use serde_json::{json, Value};

use super::targets::{self, Export, Managed};
use crate::documents::generation::DocumentFormat;

fn offered(payload: &Value) -> Vec<String> {
    targets::of(payload).unwrap().formats
}

/// A Note document the engine can read, which is what admission answers about.
fn note(text: &str) -> Value {
    json!({"kind":"note","title":"Notes","tiptapContent":{"type":"doc","content":[
        {"type":"paragraph","content":[{"type":"text","text":text}]}
    ]}})
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
        offered(&note("Decision")),
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

/// Every serialization reads the same authoring model, so content outside the authoring contract
/// is unavailable in all of them and says so where the formats were asked for.
#[test]
fn a_document_the_engine_cannot_read_offers_no_format_and_says_why() {
    let payload = json!({"kind":"note","title":"D","tiptapContent":{"type":"doc","content":[
        {"type":"mermaid","attrs":{"diagram":"graph TD;"}}
    ]}});
    let formats = targets::of(&payload).unwrap();
    assert!(formats.formats.is_empty());
    assert!(formats
        .unavailable
        .is_some_and(|reason| reason.contains("mermaid")));
    assert!(targets::requested(&payload, Path::new("/out/d.docx"))
        .unwrap_err()
        .contains("mermaid"));
}

/// Markdown carries HTML - badges, centred headers - and a document is the content that markup
/// wraps, so such a document converts like any other.
#[test]
fn markdown_that_carries_html_still_reaches_every_format() {
    let payload = json!({"kind":"markdown","title":"D","text":"<div>raw</div>"});
    let formats = targets::of(&payload).unwrap();
    assert_eq!(formats.formats, ["txt", "html", "docx", "pdf", "pptx"]);
    assert!(formats.unavailable.is_none());
}

/// A text block is the same editable content as a Note, presented differently, so it reaches the
/// same formats. Its own `.txt` is the Renderer's to write, so the engine is not asked for it.
#[test]
fn a_text_block_reaches_the_document_formats_too() {
    let payload = json!({"kind":"text","title":"Notes","plainText":"Decision",
    "tiptapContent":{"type":"doc","content":[
        {"type":"paragraph","content":[{"type":"text","text":"Decision"}]}
    ]}});
    assert_eq!(offered(&payload), ["txt", "html", "docx", "pdf", "pptx"]);
    assert_eq!(
        targets::requested(&payload, Path::new("/out/notes.pdf")).unwrap(),
        Export::Document(DocumentFormat::Pdf)
    );
}

/// Every document file reaches every serialization the engine can write. Slides are the one
/// exception: a deck is only derived from a source that carries slides.
#[test]
fn document_files_keep_their_own_bytes_and_reach_the_engine_formats() {
    let office = |extension: &str| {
        json!({"kind":"file","title":"Report","fileName":format!("report.{extension}"),
               "assetPath":format!("/owned/report.{extension}"),"sourceRevision":"sha256:current"})
    };
    for (extension, expected) in [
        ("doc", vec!["doc", "txt", "html", "docx", "pdf"]),
        ("docx", vec!["docx", "txt", "html", "pdf"]),
        ("xls", vec!["xls", "txt", "html", "docx", "pdf"]),
        ("xlsx", vec!["xlsx", "txt", "html", "docx", "pdf"]),
        ("pptx", vec!["pptx", "txt", "html", "docx", "pdf"]),
    ] {
        let payload = office(extension);
        assert_eq!(offered(&payload), expected);
        assert_eq!(
            targets::requested(&payload, Path::new(&format!("/out/report.{extension}"))).unwrap(),
            Export::Managed(Managed::Original),
        );
    }
    assert_eq!(
        offered(&office("pdf")),
        ["pdf", "txt", "html", "docx", "pptx"]
    );
    assert_eq!(
        offered(&office("ppt")),
        ["ppt", "txt", "html", "docx", "pdf", "pptx"]
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
    let processing = json!({
        "kind":"image", "fileName":"cover.png", "assetPath":"/owned/cover.png",
        "processingState":"processing"
    });
    let formats = targets::of(&processing).unwrap();
    assert!(formats.formats.is_empty());
    assert_eq!(
        formats.unavailable.as_deref(),
        Some("Wait for this resource to finish before exporting it")
    );
    assert!(targets::requested(&processing, Path::new("/out/cover.png"))
        .unwrap_err()
        .contains("Wait for this resource"));
    assert!(targets::of(
        &json!({"kind":"markdown","title":"D","text":"","derivationState":"pending"})
    )
    .unwrap()
    .unavailable
    .is_some_and(|reason| reason.contains("Wait for this resource")));
}

/// A website summary or an object is serialized by the Renderer from content it
/// already holds, so the engine truthfully offers nothing rather than failing.
#[test]
fn renderer_owned_content_is_offered_no_engine_format() {
    let formats = targets::of(&json!({"kind":"website","title":"Page"})).unwrap();
    assert!(formats.formats.is_empty());
    assert!(
        formats.unavailable.is_none(),
        "nothing is wrong with a resource whose kind simply has no engine format"
    );
}

#[test]
fn a_destination_without_an_extension_names_no_format() {
    assert!(
        targets::requested(&note("Decision"), Path::new("/out/notes"))
            .unwrap_err()
            .contains("needs a file extension")
    );
}
