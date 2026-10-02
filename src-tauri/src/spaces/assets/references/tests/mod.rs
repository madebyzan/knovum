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
    rewrite_node_asset_paths(&mut value, |_| Ok("knovum-package-asset:token".to_owned())).unwrap();
    let attrs = &value["tiptapContent"]["content"][0]["attrs"];
    assert_eq!(attrs["assetPath"], "knovum-package-asset:token");
    assert_eq!(attrs["src"], "knovum-package-asset:token");
}

#[test]
fn rewrite_restores_tiptap_package_tokens() {
    let mut value = serde_json::json!({
        "tiptapContent": {"type": "image", "attrs": {
            "assetPath": "knovum-package-asset:photo", "src": "knovum-package-asset:photo"
        }}
    });
    rewrite_node_asset_paths(&mut value, |path| {
        assert_eq!(path, "knovum-package-asset:photo");
        Ok("/imported/photo.png".into())
    })
    .unwrap();
    assert_eq!(
        value["tiptapContent"]["attrs"]["assetPath"],
        "/imported/photo.png"
    );
    assert_eq!(
        value["tiptapContent"]["attrs"]["src"],
        "/imported/photo.png"
    );
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
    rewrite_node_asset_paths(&mut value, |_| Ok("knovum-package-asset:token".to_owned())).unwrap();
    assert_eq!(
        value["text"],
        "Check ![diagram](<knovum-package-asset:token>) here."
    );
}

#[test]
fn markdown_rewrite_uses_parsed_images_and_preserves_titles() {
    let mut value = serde_json::json!({
        "text": "`![code](/ignore.png)` ![nested [alt]](/chart(a).png \"Chart\") ![ref][chart]\n\n[chart]: /ref.png \"Reference\""
    });
    rewrite_node_asset_paths(&mut value, |path| {
        Ok(format!("knovum-package-asset:{path}"))
    })
    .unwrap();
    let text = value["text"].as_str().unwrap();
    assert!(text.contains("`![code](/ignore.png)`"));
    assert!(text.contains("![nested [alt]](<knovum-package-asset:/chart(a).png> \"Chart\")"));
    assert!(text.contains("![ref](<knovum-package-asset:/ref.png> \"Reference\")"));
    assert!(text.contains("[chart]: /ref.png \"Reference\""));
}

#[test]
fn markdown_asset_path_keeps_spaces_from_angle_destinations() {
    let value = serde_json::json!({"text": "![image](</data/my image.png>)"});
    let mut paths = BTreeSet::new();
    collect_node_asset_paths(&value, &mut paths);
    assert!(paths.contains("/data/my image.png"));
}

#[test]
fn remote_markdown_images_are_not_packaged() {
    let mut value = serde_json::json!({
        "text": "![remote](https://example.com/photo.png) ![local](/data/photo.png)"
    });
    let mut paths = BTreeSet::new();
    collect_node_asset_paths(&value, &mut paths);
    assert_eq!(paths, BTreeSet::from(["/data/photo.png".to_owned()]));
    rewrite_node_asset_paths(&mut value, |_| Ok("knovum-package-asset:photo".into())).unwrap();
    assert!(value["text"]
        .as_str()
        .unwrap()
        .contains("![remote](https://example.com/photo.png)"));
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
