use rmcp::ErrorData as McpError;
use serde_json::json;

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
        let node = crate::spaces::queries::knowledge_projection::load_node(
            &mut tx,
            &args.space_id,
            &args.node_id,
        )
        .await
        .map_err(mcp_error)?;
        tx.commit()
            .await
            .map_err(|error| mcp_error(error.to_string()))?;
        let asset =
            crate::spaces::assets::node_asset::resolve(&self.app, &node, &args.asset_revision)
                .await
                .map_err(mcp_error)?;
        let data = &node["data"];
        let path = asset.path;
        json_text(json!({
            "spaceId": args.space_id, "nodeId": args.node_id,
            "assetRevision": args.asset_revision, "sourceRevision": data.get("sourceRevision"),
            "path": path, "mimeType": mime_guess::from_path(&path).first_raw(),
            "representation": asset.representation,
            "usage": "Read this file as source material. Save new deliverables in the output workspace; never overwrite this managed source.",
        }))
    }
}
