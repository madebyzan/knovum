use super::*;

#[test]
fn collects_top_level_and_tiptap_assets() {
    let value = serde_json::json!({
        "assetPath": "/data/spaces/assets/file.pdf",
        "tiptapContent": {
            "type": "doc",
            "content": [
                {"type": "image", "attrs": {"assetPath": "/data/spaces/assets/editor-a.png", "src": "display-only"}},
                {"type": "image", "attrs": {"assetPath": "/data/spaces/assets/editor-b.png", "src": "ignored"}}
            ]
        }
    });
    let mut paths = BTreeSet::new();
    collect_node_asset_paths(&value, &mut paths);
    assert_eq!(paths.len(), 3);
    assert!(paths.contains("/data/spaces/assets/editor-a.png"));
    assert!(paths.contains("/data/spaces/assets/editor-b.png"));
}

#[test]
fn rewrite_updates_tiptap_canonical_and_display_references() {
    let mut value = serde_json::json!({
        "kind": "note",
        "title": "note",
        "tiptapContent": {"type": "doc", "content": [
            {"type": "image", "attrs": {"assetPath": "/data/editor.png", "src": "display-only"}}
        ]}
    });
    rewrite_node_asset_paths(&mut value, |_| "knovum-package-asset:token".to_owned());
    let attrs = &value["tiptapContent"]["content"][0]["attrs"];
    assert_eq!(attrs["assetPath"], "knovum-package-asset:token");
    assert_eq!(attrs["src"], "knovum-package-asset:token");
}

#[test]
fn collects_markdown_image_assets() {
    let value = serde_json::json!({
        "kind": "markdown",
        "title": "MD with images",
        "text": "# Header\n\n![Chart](/data/spaces/assets/chart.png)\n\nSome text with ![Photo](asset://localhost/%2Fdata%2Fspaces%2Fassets%2Fphoto.jpg) inline."
    });
    let mut paths = BTreeSet::new();
    collect_node_asset_paths(&value, &mut paths);
    assert_eq!(paths.len(), 2);
    assert!(paths.contains("/data/spaces/assets/chart.png"));
    assert!(paths.contains("/data/spaces/assets/photo.jpg"));
}

#[test]
fn rewrite_updates_markdown_image_references() {
    let mut value = serde_json::json!({
        "kind": "markdown",
        "title": "MD",
        "text": "Check ![diagram](/data/spaces/assets/diag.png) here."
    });
    rewrite_node_asset_paths(&mut value, |_| "knovum-package-asset:token".to_owned());
    assert_eq!(
        value["text"],
        "Check ![diagram](knovum-package-asset:token) here."
    );
}

#[test]
fn percent_decoding_keeps_bytes_of_invalid_escapes() {
    assert_eq!(
        asset_url::asset_path("asset://localhost/a%zzb.png"),
        "a%zzb.png"
    );
    assert_eq!(
        asset_url::asset_path("asset://localhost/100%.png"),
        "100%.png"
    );
    assert_eq!(
        asset_url::asset_path("asset://localhost/a%2Fb.png"),
        "a/b.png"
    );
    assert_eq!(
        asset_url::asset_path("asset://localhost/%E4%B8%AD%E6%96%87.png"),
        "中文.png"
    );
}
