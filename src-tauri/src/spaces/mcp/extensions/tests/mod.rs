use crate::extensions::contracts::{
    AgentExtensionSnapshot, AgentExtensionTool, AgentToolBinding, McpConnectionCatalog,
    McpToolPolicy,
};
use rmcp::model::{Prompt, Resource};
use serde_json::json;

fn snapshot() -> AgentExtensionSnapshot {
    let tool = |name: &str, binding| AgentExtensionTool {
        name: name.into(),
        description: String::new(),
        // Projection must use the frozen binding even when the wire Schema changes shape.
        input_schema: json!({"type":"object"}),
        binding,
        read_only: true,
        policy: McpToolPolicy::Confirm,
    };
    AgentExtensionSnapshot {
        plugins: Vec::new(),
        id: "snapshot".into(),
        digest: "reviewed".into(),
        skills: vec![],
        files: vec![],
        skill_tool_bindings: Default::default(),
        connectors: vec![],
        connections: vec![],
        accounts: vec![],
        tools: vec![
            tool(
                "resource",
                AgentToolBinding::McpReadResource {
                    connection_id: "connection".into(),
                    resource_templates: vec![],
                    resource_uris: vec![
                        "fixture://selected".into(),
                        "fixture://expanded/item".into(),
                    ],
                },
            ),
            tool(
                "prompt",
                AgentToolBinding::McpGetPrompt {
                    connection_id: "connection".into(),
                    prompt_names: vec!["selected".into()],
                },
            ),
        ],
        catalogs: [(
            "connection".into(),
            McpConnectionCatalog {
                server: json!({}),
                tools: vec![],
                resources: vec![
                    Resource::new("fixture://selected", "Selected resource"),
                    Resource::new("fixture://unselected", "Unselected resource"),
                ],
                resource_templates: vec![],
                prompts: vec![
                    Prompt::new("selected", Some("Selected prompt"), None),
                    Prompt::new("unselected", None::<String>, None),
                ],
                digest: "catalog".into(),
            },
        )]
        .into_iter()
        .collect(),
    }
}

#[test]
fn resource_catalog_uses_selected_bindings_and_preserves_sdk_metadata() {
    let snapshot = snapshot();
    let resources = super::catalog_projection::selected_resources(&snapshot);
    assert_eq!(resources.len(), 2);
    assert_eq!(resources[0].uri, "fixture://selected");
    assert_eq!(resources[0].declaration.name, "Selected resource");
    assert!(resources[0]
        .declaration
        .uri
        .starts_with("knovum-mcp-resource://connection/"));
    assert_eq!(resources[1].uri, "fixture://expanded/item");
    assert_eq!(resources[1].declaration.name, "fixture://expanded/item");
}

#[test]
fn prompt_catalog_uses_selected_bindings_without_reading_wire_schema() {
    let snapshot = snapshot();
    let prompts = super::catalog_projection::selected_prompts(&snapshot);
    assert_eq!(prompts.len(), 1);
    assert_eq!(prompts[0].name, "selected");
    assert_eq!(
        prompts[0].declaration.description.as_deref(),
        Some("Selected prompt")
    );
    assert!(prompts[0].declaration.name.starts_with("prompt_"));
}
