
use super::web_media_preview;

#[test]
fn web_downloaded_thumbnail_uses_space_asset_authority() {
    assert!(web_media_preview(
        &serde_json::json!({"origin": {"kind": "web"}})
    ));
    assert!(!web_media_preview(&serde_json::json!({"kind": "video"})));
}
