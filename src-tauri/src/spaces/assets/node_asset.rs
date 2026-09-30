use std::path::PathBuf;

use serde::Serialize;
use serde_json::Value;

#[derive(Clone, Copy, PartialEq, Eq, Serialize)]
pub(crate) enum AssetRepresentation {
    #[serde(rename = "original-asset")]
    Original,
    #[serde(rename = "current-render")]
    Rendered,
}

pub(crate) struct NodeAsset {
    pub path: PathBuf,
    pub representation: AssetRepresentation,
}

pub(crate) async fn resolve(
    app: &tauri::AppHandle,
    node: &Value,
    expected_revision: &str,
) -> Result<NodeAsset, String> {
    require_revision(node, expected_revision)?;
    let data = &node["data"];
    let edited =
        node["kind"] == "image" && data.get("imageEdit").is_some_and(|edit| !edit.is_null());
    let (path, representation) = if edited {
        (
            super::image::rendered_preview(app, data).await?,
            AssetRepresentation::Rendered,
        )
    } else {
        let source = data["assetPath"]
            .as_str()
            .ok_or("node-has-no-asset: this resource has no local source file")?;
        (
            super::authorize_spaces_asset(app, std::path::Path::new(source))?,
            AssetRepresentation::Original,
        )
    };
    if !path.is_file() {
        return Err("node-asset-unavailable: source is not a regular file".into());
    }
    Ok(NodeAsset {
        path,
        representation,
    })
}

fn require_revision(node: &Value, expected: &str) -> Result<(), String> {
    let revision = node["assetRevision"]
        .as_str()
        .ok_or("asset-revision-unavailable: canvas resource has no published asset")?;
    if revision != expected {
        return Err(
            "asset-revision-changed: read the current node before requesting its source file"
                .into(),
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn asset_identity_does_not_substitute_knowledge_or_original_file_revisions() {
        let node = serde_json::json!({"assetRevision":"rendered1:current","knowledgeRevision":"node1:content","sourceRevision":"sha256:file"});
        assert!(super::require_revision(&node, "rendered1:current").is_ok());
        for revision in ["rendered1:old", "node1:content", "sha256:file"] {
            assert!(super::require_revision(&node, revision).is_err());
        }
        assert!(super::require_revision(
            &serde_json::json!({"sourceRevision":"sha256:file"}),
            "sha256:file"
        )
        .is_err());
    }
}
