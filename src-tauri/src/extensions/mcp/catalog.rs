use super::{
    super::{content, contracts::McpConnectionCatalog},
    client::ExtensionClient,
};
use rmcp::{service::RunningService, RoleClient};
use serde_json::Value;

pub(super) async fn read(
    service: &RunningService<RoleClient, ExtensionClient>,
) -> Result<McpConnectionCatalog, String> {
    let info = service
        .peer_info()
        .ok_or("mcp-server-information-missing")?;
    let catalog = tokio::time::timeout(std::time::Duration::from_secs(30), async {
        let tools = if info.capabilities.tools.is_some() {
            pages(|params| async move {
                let page = service.list_tools(params).await?;
                Ok((page.tools, page.next_cursor))
            })
            .await?
            .into_iter()
            .map(tool)
            .collect::<Result<Vec<_>, _>>()?
        } else {
            Vec::new()
        };
        let resources = if info.capabilities.resources.is_some() {
            pages(|params| async move {
                let page = service.list_resources(params).await?;
                Ok((page.resources, page.next_cursor))
            })
            .await?
        } else {
            Vec::new()
        };
        let resource_templates = if info.capabilities.resources.is_some() {
            pages(|params| async move {
                let page = service.list_resource_templates(params).await?;
                Ok((page.resource_templates, page.next_cursor))
            })
            .await?
        } else {
            Vec::new()
        };
        let prompts = if info.capabilities.prompts.is_some() {
            pages(|params| async move {
                let page = service.list_prompts(params).await?;
                Ok((page.prompts, page.next_cursor))
            })
            .await?
        } else {
            Vec::new()
        };
        Ok::<_, String>((tools, resources, resource_templates, prompts))
    })
    .await
    .map_err(|_| "mcp-catalog-timeout")??;
    let (tools, resources, resource_templates, prompts) = catalog;
    unique(tools.iter().map(|tool| tool.name.as_str()))?;
    unique(resources.iter().map(|resource| resource.uri.as_str()))?;
    unique(
        resource_templates
            .iter()
            .map(|template| template.uri_template.as_str()),
    )?;
    unique(prompts.iter().map(|prompt| prompt.name.as_str()))?;
    let server = serde_json::to_value(info.as_ref()).map_err(|error| error.to_string())?;
    let encoded = serde_json::to_vec(&(&server, &tools, &resources, &resource_templates, &prompts))
        .map_err(|error| error.to_string())?;
    if encoded.len() > 8 * 1024 * 1024 {
        return Err("mcp-catalog-size-limit".into());
    }
    Ok(McpConnectionCatalog {
        server,
        tools,
        resources,
        resource_templates,
        prompts,
        digest: content::digest(&encoded),
    })
}

fn unique<'a>(identities: impl Iterator<Item = &'a str>) -> Result<(), String> {
    let mut seen = std::collections::BTreeSet::new();
    for identity in identities {
        if identity.is_empty() || identity.len() > 8192 || !seen.insert(identity) {
            return Err("mcp-catalog-identity-empty-oversized-or-duplicated".into());
        }
    }
    Ok(())
}

fn tool(tool: rmcp::model::Tool) -> Result<super::super::contracts::McpCatalogTool, String> {
    let input_schema = Value::Object((*tool.input_schema).clone());
    if input_schema["type"] != "object" {
        return Err("mcp-tool-input-schema-must-describe-object".into());
    }
    jsonschema::validator_for(&input_schema).map_err(|_| "mcp-tool-input-schema-invalid")?;
    let output_schema = tool
        .output_schema
        .map(|schema| Value::Object((*schema).clone()));
    if let Some(schema) = &output_schema {
        jsonschema::validator_for(schema).map_err(|_| "mcp-tool-output-schema-invalid")?;
    }
    let schema_digest = content::digest(
        &serde_json::to_vec(&(&input_schema, &output_schema)).map_err(|error| error.to_string())?,
    );
    Ok(super::super::contracts::McpCatalogTool {
        name: tool.name.into_owned(),
        description: tool
            .description
            .map(|description| description.into_owned())
            .unwrap_or_default(),
        input_schema,
        output_schema,
        schema_digest,
        read_only_hint: tool
            .annotations
            .and_then(|annotations| annotations.read_only_hint),
    })
}

async fn pages<T: serde::Serialize, F, Fut>(mut request: F) -> Result<Vec<T>, String>
where
    F: FnMut(Option<rmcp::model::PaginatedRequestParams>) -> Fut,
    Fut:
        std::future::Future<Output = Result<(Vec<T>, Option<String>), rmcp::service::ServiceError>>,
{
    let mut cursor = None;
    let mut cursors = std::collections::BTreeSet::new();
    let mut tools = Vec::new();
    let mut bytes = 0;
    for _ in 0..100 {
        let (page, next) = request(cursor.map(|cursor| {
            rmcp::model::PaginatedRequestParams::default().with_cursor(Some(cursor))
        }))
        .await
        .map_err(|_| "mcp-catalog-request-failed")?;
        bytes += serde_json::to_vec(&page)
            .map_err(|error| error.to_string())?
            .len();
        if tools.len() + page.len() > 4096 || bytes > 8 * 1024 * 1024 {
            return Err("mcp-catalog-budget-exceeded".into());
        }
        tools.extend(page);
        let Some(next) = next else {
            return Ok(tools);
        };
        if !cursors.insert(next.clone()) {
            return Err("mcp-catalog-repeated-cursor".into());
        }
        cursor = Some(next);
    }
    Err("mcp-catalog-pagination-limit".into())
}
