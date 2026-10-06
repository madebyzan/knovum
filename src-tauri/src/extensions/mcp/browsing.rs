use super::super::{
    content,
    contracts::{McpResourceContent, McpResourceSelection},
    database,
};
use rmcp::model::{GetPromptRequestParams, ReadResourceRequestParams};
use serde_json::Value;
use tauri::AppHandle;

pub(crate) async fn read_resource(
    app: &AppHandle,
    id: &str,
    revision: &str,
    catalog_digest: &str,
    selection: &McpResourceSelection,
) -> Result<McpResourceContent, String> {
    let connection = super::connections::load(&database::pool(app).await?, id).await?;
    if connection.revision != revision {
        return Err("mcp-browse-connection-revised".into());
    }
    super::connections::require_current(app, &connection).await?;
    let connector = super::super::connectors::definitions::load(
        &database::pool(app).await?,
        &connection.connector_id,
    )
    .await?;
    let session = super::client::connect(app, &connection, &connector, None).await?;
    let read = async {
        let catalog = super::catalog::read(&session.service).await?;
        if catalog.digest != catalog_digest {
            return Err("mcp-browse-catalog-revised".into());
        }
        let uri = match selection {
            McpResourceSelection::Resource { uri } => {
                if !catalog
                    .resources
                    .iter()
                    .any(|resource| &resource.uri == uri)
                {
                    return Err("mcp-resource-not-in-catalog".into());
                }
                uri.clone()
            }
            McpResourceSelection::Template { selection } => {
                super::resource_selection::expand(&catalog, selection)?
            }
        };
        super::connections::require_current(app, &connection).await?;
        let response = tokio::time::timeout(
            std::time::Duration::from_secs(120),
            session
                .service
                .read_resource(ReadResourceRequestParams::new(uri.clone())),
        )
        .await
        .map_err(|_| "mcp-resource-read-timeout")?
        .map_err(|_| "mcp-resource-read-failed")?;
        let encoded = serde_json::to_vec(&response).map_err(|error| error.to_string())?;
        if encoded.len() > 8 * 1024 * 1024 {
            return Err("mcp-resource-content-exceeds-8-mib".into());
        }
        let contents = response.contents;
        Ok(McpResourceContent {
            connection_id: id.into(),
            connection_revision: revision.into(),
            catalog_digest: catalog_digest.into(),
            uri,
            contents,
            digest: content::digest(&encoded),
        })
    }
    .await;
    session.close().await;
    read
}

#[tauri::command]
pub async fn read_mcp_resource(
    app: AppHandle,
    id: String,
    revision: String,
    catalog_digest: String,
    selection: McpResourceSelection,
) -> Result<McpResourceContent, String> {
    read_resource(&app, &id, &revision, &catalog_digest, &selection).await
}

#[tauri::command]
pub async fn get_mcp_prompt(
    app: AppHandle,
    id: String,
    revision: String,
    catalog_digest: String,
    name: String,
    arguments: serde_json::Map<String, Value>,
) -> Result<Value, String> {
    let connection = super::connections::load(&database::pool(&app).await?, &id).await?;
    if connection.revision != revision {
        return Err("mcp-browse-connection-revised".into());
    }
    super::connections::require_current(&app, &connection).await?;
    let connector = super::super::connectors::definitions::load(
        &database::pool(&app).await?,
        &connection.connector_id,
    )
    .await?;
    let session = super::client::connect(&app, &connection, &connector, None).await?;
    let read = async {
        let catalog = super::catalog::read(&session.service).await?;
        if catalog.digest != catalog_digest
            || !catalog.prompts.iter().any(|prompt| prompt.name == name)
        {
            return Err("mcp-browse-prompt-not-in-reviewed-catalog".into());
        }
        let schema =
            super::resource_selection::prompt_schema(&catalog, std::slice::from_ref(&name));
        jsonschema::validator_for(&schema)
            .map_err(|error| error.to_string())?
            .validate(&serde_json::json!({"name":name,"arguments":arguments}))
            .map_err(|error| error.to_string())?;
        super::connections::require_current(&app, &connection).await?;
        let response = tokio::time::timeout(
            std::time::Duration::from_secs(120),
            session
                .service
                .get_prompt(GetPromptRequestParams::new(name).with_arguments(arguments)),
        )
        .await
        .map_err(|_| "mcp-prompt-read-timeout")?
        .map_err(|_| "mcp-prompt-read-failed")?;
        if serde_json::to_vec(&response)
            .map_err(|error| error.to_string())?
            .len()
            > 8 * 1024 * 1024
        {
            return Err("mcp-prompt-content-exceeds-8-mib".into());
        }
        serde_json::to_value(response).map_err(|error| error.to_string())
    }
    .await;
    session.close().await;
    read
}
