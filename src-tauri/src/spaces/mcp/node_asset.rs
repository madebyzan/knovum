use rmcp::ErrorData as McpError;
use serde_json::{json, Value};

use super::{arguments::AssetArgs, mcp_error, support::json_text, SpaceMcpServer};

impl SpaceMcpServer {
    pub(super) async fn node_asset(&self, args: AssetArgs) -> Result<String, McpError> {
        self.require_read(&args.space_id).await?;
        let pool = crate::spaces::db::pool(&self.app)
            .await
            .map_err(mcp_error)?;
        let mut tx = pool
            .begin()
            .await
            .map_err(|error| mcp_error(error.to_string()))?;
        let node = crate::spaces::queries::knowledge_projection::get_node(
            &mut tx,
            &args.space_id,
            &args.node_id,
        )
        .await
        .map_err(mcp_error)?;
        tx.commit()
            .await
            .map_err(|error| mcp_error(error.to_string()))?;
        let data = &node["data"];
        let revision = require_asset_revision(&node, &args.asset_revision)?;
        let edited =
            node["kind"] == "image" && data.get("imageEdit").is_some_and(|edit| !edit.is_null());
        let path = if edited {
            crate::spaces::assets::image::rendered_preview(&self.app, data)
                .await
                .map_err(mcp_error)?
        } else {
            let source = data["assetPath"].as_str().ok_or_else(|| {
                mcp_error("node-has-no-asset: this resource has no local source file")
            })?;
            crate::spaces::assets::authorize_spaces_asset(&self.app, std::path::Path::new(source))
                .map_err(mcp_error)?
        };
        json_text(json!({
            "spaceId": args.space_id, "nodeId": args.node_id,
            "assetRevision": revision, "sourceRevision": data.get("sourceRevision"),
            "path": path, "mimeType": mime_guess::from_path(&path).first_raw(),
            "representation": if edited { "current-render" } else { "original-asset" },
            "usage": "Read this file as source material. Save new deliverables in the output workspace; never overwrite this managed source.",
        }))
    }
}

fn require_asset_revision<'a>(node: &'a Value, expected: &str) -> Result<&'a str, McpError> {
    let revision = node["assetRevision"].as_str().ok_or_else(|| {
        mcp_error("asset-revision-unavailable: canvas resource has no published asset")
    })?;
    if revision != expected {
        return Err(McpError::invalid_params(
            "asset-revision-changed: read the current node before requesting its source file",
            None,
        ));
    }
    Ok(revision)
}

#[cfg(test)]
mod tests {
    #[test]
    fn source_files_are_pinned_to_rendered_identity_without_substituting_index_revisions() {
        let node = serde_json::json!({"assetRevision":"rendered1:current","knowledgeRevision":"node1:content","sourceRevision":"sha256:file"});
        assert_eq!(
            super::require_asset_revision(&node, "rendered1:current").unwrap(),
            "rendered1:current"
        );
        assert!(super::require_asset_revision(&node, "rendered1:old").is_err());
        assert!(super::require_asset_revision(&node, "node1:content").is_err());
        assert!(super::require_asset_revision(&node, "sha256:file").is_err());
        assert!(super::require_asset_revision(
            &serde_json::json!({"sourceRevision":"sha256:file"}),
            "sha256:file"
        )
        .is_err());
    }
}
