use super::{apply, authoritative, validate_declared};
use serde_json::json;

#[test]
fn integer_and_decimal_json_numbers_share_the_webview_representation() {
    assert_eq!(
        super::canonical::stable_json(&json!(12)),
        super::canonical::stable_json(&json!(12.0))
    );
}

#[test]
fn local_tiptap_paths_do_not_change_revision() {
    let left = json!({"kind":"note","title":"N","tiptapContent":{"type":"doc","content":[{"type":"image","attrs":{"src":"asset://a","assetPath":"/a","alt":"x"}}]},"plainText":""});
    let right = json!({"kind":"note","title":"N","tiptapContent":{"type":"doc","content":[{"type":"image","attrs":{"src":"asset://b","assetPath":"/b","alt":"x"}}]},"plainText":""});
    assert_eq!(
        authoritative(&left).unwrap(),
        authoritative(&right).unwrap()
    );
}

#[test]
fn remote_image_src_remains_semantic() {
    let left = json!({"kind":"note","title":"N","tiptapContent":{"type":"doc","content":[{"type":"image","attrs":{"src":"https://a.example/image.png","alt":"x"}}]},"plainText":""});
    let right = json!({"kind":"note","title":"N","tiptapContent":{"type":"doc","content":[{"type":"image","attrs":{"src":"https://b.example/image.png","alt":"x"}}]},"plainText":""});
    assert_ne!(
        authoritative(&left).unwrap(),
        authoritative(&right).unwrap()
    );
}

#[test]
fn non_image_src_remains_semantic() {
    let left = json!({"kind":"note","title":"N","tiptapContent":{"type":"doc","content":[{"type":"embed","attrs":{"src":"https://a"}}]},"plainText":""});
    let right = json!({"kind":"note","title":"N","tiptapContent":{"type":"doc","content":[{"type":"embed","attrs":{"src":"https://b"}}]},"plainText":""});
    assert_ne!(
        authoritative(&left).unwrap(),
        authoritative(&right).unwrap()
    );
}

#[test]
fn numbers_use_formatter_independent_ieee754_bits() {
    let large = json!({
        "kind":"object", "title":"N", "tags":[], "objectTypeId":"system:test",
        "summary":"", "status":"draft", "properties":{"n":1e20}
    });
    assert_eq!(
        authoritative(&large).unwrap().as_deref(),
        Some("node1:ecbb9a3affd2948d")
    );
    let negative_zero = json!({
        "kind":"object", "title":"N", "tags":[], "objectTypeId":"system:test",
        "summary":"", "status":"draft", "properties":{"n":-0.0}
    });
    let positive_zero = json!({
        "kind":"object", "title":"N", "tags":[], "objectTypeId":"system:test",
        "summary":"", "status":"draft", "properties":{"n":0.0}
    });
    assert_eq!(
        authoritative(&negative_zero).unwrap(),
        authoritative(&positive_zero).unwrap()
    );
}

#[test]
fn non_bmp_object_keys_match_utf8_canonical_order() {
    let data = json!({
        "kind":"object", "title":"O", "tags":[], "objectTypeId":"system:test",
        "summary":"", "status":"draft", "properties":{"𐀀":"a","":"b"}
    });
    assert_eq!(
        authoritative(&data).unwrap().as_deref(),
        Some("node1:526d1abe2260909e")
    );
}

#[test]
fn declared_revision_must_match_content() {
    let mut data = json!({"kind":"markdown","title":"M","text":"alpha"});
    apply(&mut data).unwrap();
    assert!(validate_declared(&data).is_ok());
    data["text"] = json!("beta");
    assert!(validate_declared(&data).is_err());
}

#[test]
fn free_canvas_text_has_no_knowledge_revision() {
    let text = json!({
        "kind":"text", "title":"Label", "plainText":"Label",
        "tiptapContent":{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Label"}]}]}
    });
    assert_eq!(authoritative(&text).unwrap(), None);
}

#[test]
fn media_kind_is_authoritative_and_apply_declares_it() {
    let video = json!({
        "kind":"video", "title":"Media", "tags":[], "fileName":"media.mp4",
        "origin":{"kind":"web","url":"https://example.com/watch/1","provider":"Example","externalId":"1"}
    });
    let mut audio = video.clone();
    audio["kind"] = json!("audio");
    assert_ne!(
        authoritative(&video).unwrap(),
        authoritative(&audio).unwrap()
    );
    apply(&mut audio).unwrap();
    assert!(validate_declared(&audio).is_ok());
}

#[test]
fn media_source_revision_is_authoritative() {
    let first = json!({
        "kind":"video", "title":"Media", "tags":[], "fileName":"media.mp4",
        "origin":{"kind":"web","url":"https://example.com/watch/1","provider":"Example","externalId":"1"},
        "sourceRevision":"sha256:first"
    });
    let mut second = first.clone();
    second["sourceRevision"] = json!("sha256:second");
    assert_ne!(
        authoritative(&first).unwrap(),
        authoritative(&second).unwrap()
    );
}

#[test]
fn media_origin_is_authoritative() {
    let data = json!({
        "kind":"audio", "fileName":"track.m4a",
        "title":"Track",
        "creator":"Artist",
        "origin":{"kind":"web","url":"https://soundcloud.com/a/b","provider":"Soundcloud","externalId":"42"}
    });
    let mut changed = data.clone();
    changed["origin"]["externalId"] = json!("43");
    assert_ne!(
        authoritative(&data).unwrap(),
        authoritative(&changed).unwrap()
    );
}

#[test]
fn transcript_edits_change_media_knowledge_revision() {
    let data = json!({"kind":"video","title":"V","sourceRevision":"asset:1","mediaKnowledge":{"sourceRevision":"asset:1","transcript":{"subtitleCues":[{"startMs":0,"endMs":1000,"text":"Original"}]}}});
    let mut edited = data.clone();
    edited["mediaKnowledge"]["transcript"]["subtitleCues"][0]["text"] = json!("Edited");
    assert_ne!(
        authoritative(&data).unwrap(),
        authoritative(&edited).unwrap()
    );
}
#[test]
fn image_asset_and_edit_are_part_of_derivation_knowledge_identity() {
    let data = json!({"kind":"image","title":"I","sourceRevision":"asset:1"});
    let mut replaced = data.clone();
    replaced["sourceRevision"] = json!("asset:2");
    let mut edited = data.clone();
    edited["imageEdit"] = json!({"rotation":90});
    assert_ne!(
        authoritative(&data).unwrap(),
        authoritative(&replaced).unwrap()
    );
    assert_ne!(
        authoritative(&data).unwrap(),
        authoritative(&edited).unwrap()
    );
}
