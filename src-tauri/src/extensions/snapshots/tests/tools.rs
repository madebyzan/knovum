use crate::extensions::contracts::*;
use serde_json::json;
fn fixture() -> (
    McpConnection,
    McpConnectionCatalog,
    AgentConnectionSelection,
) {
    let tool = McpCatalogTool {
        name: "publish".into(),
        description: "Publish externally".into(),
        input_schema: json!({"type":"object","properties":{}}),
        output_schema: None,
        schema_digest: "reviewed".into(),
        read_only_hint: Some(true),
    };
    let connection = McpConnection {
        id: "connection".into(),
        connector_id: "connector".into(),
        name: "Fixture".into(),
        plugin_id: None,
        component_name: None,
        revision: "revision".into(),
        endpoint: McpEndpoint::StreamableHttp {
            url: "https://example.com/mcp".into(),
            headers: Default::default(),
            allow_local_network: false,
        },
        account_id: Some("selected-account".into()),
        enabled: true,
        tools: vec![McpToolPermission {
            name: tool.name.clone(),
            schema_digest: tool.schema_digest.clone(),
            read_only: false,
            policy: McpToolPolicy::Confirm,
        }],
    };
    let catalog = McpConnectionCatalog {
        server: json!({}),
        tools: vec![tool],
        resources: vec![serde_json::from_value(
            json!({"uri":"fixture://selected","name":"Selected"}),
        )
        .unwrap()],
        resource_templates: vec![],
        prompts: vec![serde_json::from_value(
            json!({"name":"draft","arguments":[{"name":"subject","required":true}]}),
        )
        .unwrap()],
        digest: "catalog".into(),
    };
    let selection = AgentConnectionSelection {
        connection_id: connection.id.clone(),
        tool_names: vec!["publish".into()],
        resource_uris: vec!["fixture://selected".into()],
        resource_templates: vec![],
        prompt_names: vec!["draft".into()],
    };
    (connection, catalog, selection)
}
#[test]
fn host_review_controls_read_only_projection_and_selected_identity() {
    let (connection, catalog, selection) = fixture();
    let read_only = super::tools::project(&connection, catalog.clone(), true, &selection).unwrap();
    assert_eq!(read_only.len(), 2);
    assert!(read_only.iter().all(|tool| tool.read_only));
    assert!(read_only
        .iter()
        .all(|tool| !matches!(tool.binding, AgentToolBinding::McpTool { .. })));
    let execute = super::tools::project(&connection, catalog, false, &selection).unwrap();
    assert!(
        matches!(&execute[0].binding,AgentToolBinding::McpTool{ account_id:Some(id),.. } if id=="selected-account")
    );
    assert_eq!(execute[0].policy, McpToolPolicy::Confirm);
}
#[test]
fn changed_schema_and_unlisted_resources_cannot_expand_selection() {
    let (mut connection, catalog, mut selection) = fixture();
    connection.tools[0].schema_digest = "other".into();
    assert_eq!(
        super::tools::project(&connection, catalog.clone(), false, &selection).unwrap_err(),
        "mcp-tool-schema-changed-review-permissions: publish"
    );
    selection
        .resource_uris
        .push("file:///private/unselected".into());
    assert_eq!(
        super::tools::project(&connection, catalog, false, &selection).unwrap_err(),
        "mcp-resource-not-in-catalog"
    );
}
#[test]
fn resource_and_prompt_schemas_bind_exact_user_selected_content() {
    let (connection, catalog, selection) = fixture();
    let projected = super::tools::project(&connection, catalog, true, &selection).unwrap();
    let resource = jsonschema::validator_for(&projected[0].input_schema).unwrap();
    assert!(matches!(&projected[0].binding,
        AgentToolBinding::McpReadResource { connection_id, resource_uris }
        if connection_id == "connection" && resource_uris == &selection.resource_uris));
    assert!(resource.is_valid(&json!({"uri":"fixture://selected"})));
    assert!(!resource.is_valid(&json!({"uri":"fixture://unselected"})));
    let prompt = jsonschema::validator_for(&projected[1].input_schema).unwrap();
    assert!(matches!(&projected[1].binding,
        AgentToolBinding::McpGetPrompt { connection_id, prompt_names }
        if connection_id == "connection" && prompt_names == &selection.prompt_names));
    assert!(prompt.is_valid(&json!({"name":"draft","arguments":{"subject":"Reviewed"}})));
    assert!(!prompt.is_valid(&json!({"name":"draft"})));
    assert!(!prompt.is_valid(&json!({"name":"other","arguments":{"subject":"Reviewed"}})));
}

#[test]
fn extension_selection_has_one_connection_identity_and_rejects_obsolete_payloads() {
    let (_, _, connection) = fixture();
    let selected = json!({"skillIds":[],"connections":[connection]});
    let selection: AgentExtensionSelection = serde_json::from_value(selected.clone()).unwrap();
    assert_eq!(selection.connections[0].connection_id, "connection");
    let mut obsolete = selected;
    obsolete["connectionIds"] = json!(["other"]);
    assert!(serde_json::from_value::<AgentExtensionSelection>(obsolete).is_err());
}
