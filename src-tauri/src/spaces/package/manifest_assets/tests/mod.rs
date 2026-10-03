use std::collections::HashSet;

use serde_json::{json, Value};

use super::{parse, ManifestAssets};

fn digest(fill: char) -> String {
    format!("sha256:{}", fill.to_string().repeat(64))
}

fn parse_node(data: Value) -> Result<ManifestAssets, String> {
    parse(
        &json!({
            "nodes": [{ "data": data }],
            "layouts": [],
            "voiceRevisions": [],
            "speechProjectRevisions": [],
            "speechRenders": []
        })
        .to_string(),
    )
}

#[test]
fn collects_tokens_from_every_packaged_reference() {
    let data = json!({
        "kind": "markdown",
        "text": "![Chart](knovum-package-asset:markdown-token)",
        "tiptapContent": {"type": "doc", "content": [
            {"type": "image", "attrs": {
                "assetPath": "knovum-package-asset:inline-token",
                "src": "knovum-package-asset:inline-token"
            }}
        ]},
        "faviconPath": "knovum-package-asset:favicon-token"
    });
    let assets = parse_node(data).expect("markdown references are packaged too");
    assert_eq!(
        assets.used_tokens,
        ["markdown-token", "inline-token", "favicon-token"]
            .into_iter()
            .map(str::to_owned)
            .collect::<HashSet<_>>()
    );
    assert!(assets.source_revisions.is_empty());
}

#[test]
fn only_the_managed_asset_must_match_the_node_digest() {
    let revision = digest('0');
    let data = json!({
        "kind": "file",
        "assetPath": "knovum-package-asset:managed-token",
        "previewImagePath": "knovum-package-asset:preview-token",
        "sourceRevision": revision
    });
    let assets = parse_node(data).expect("a managed asset carries its node digest");
    assert_eq!(assets.source_revisions["managed-token"], revision);
    assert_eq!(assets.source_revisions.len(), 1);
    assert!(assets.used_tokens.contains("preview-token"));
}
