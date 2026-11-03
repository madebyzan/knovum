use super::*;

#[test]
fn a_web_downloaded_thumbnail_uses_space_asset_authority() {
    assert!(web_media_preview(
        &serde_json::json!({"origin": {"kind": "web"}})
    ));
    assert!(!web_media_preview(&serde_json::json!({"kind": "video"})));
}
