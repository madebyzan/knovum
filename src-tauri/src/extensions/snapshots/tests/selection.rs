use super::super::selection::{require_package, resolve};
use crate::extensions::contracts::{
    AgentExtensionSelection, AgentExtensionSnapshot, ConnectorAuthorization, ConnectorDefinition,
    McpConnection, McpEndpoint, McpToolPermission, McpToolPolicy, PluginComponent,
    PluginConnectorDeclaration, PluginInstallation, SkillRevision,
};
use serde_json::json;

fn plugin() -> PluginInstallation {
    serde_json::from_value(json!({
        "id":"package", "enabled":true, "builtin":true, "publication":null,
        "source":{"type":"directory","path":"/package"},
        "revision":{
            "digest":"one", "sourceRevision":"one", "fileCount":2, "byteLength":2,
            "components":[
                {"name":"research","kind":"skill","supported":true,"diagnostic":null},
                {"name":"delivery","kind":"skill","supported":true,"diagnostic":null}
            ], "connectors":[],
            "manifest":{"$schema":"https://test/plugin.schema.json","name":"Knowledge"}
        }
    }))
    .unwrap()
}
fn skill(name: &str) -> SkillRevision {
    SkillRevision {
        id: name.into(),
        name: name.into(),
        plugin_id: Some("package".into()),
        description: name.into(),
        digest: "one".into(),
        enabled: true,
        requirements: vec![],
        included_tools: vec![],
        unsupported_requirements: vec![],
    }
}
fn selection() -> AgentExtensionSelection {
    AgentExtensionSelection {
        plugin_ids: vec!["package".into()],
        skill_ids: vec![],
        connections: vec![],
    }
}

#[test]
fn package_selection_expands_all_skills_and_freezes_package_identity() {
    let (resolved, packages) = resolve(
        &selection(),
        &[plugin()],
        &[skill("research"), skill("delivery")],
        &[],
        &[],
    )
    .unwrap();
    assert_eq!(resolved.skill_ids, vec!["research", "delivery"]);
    assert_eq!(packages[0].revision.digest, "one");
}

#[test]
fn direct_packaged_skill_selection_is_rejected_instead_of_partially_loading() {
    let selected = AgentExtensionSelection {
        plugin_ids: vec![],
        skill_ids: vec!["research".into()],
        connections: vec![],
    };
    assert_eq!(
        resolve(
            &selected,
            &[plugin()],
            &[skill("research"), skill("delivery")],
            &[],
            &[]
        )
        .unwrap_err(),
        "plugin-components-require-package-selection"
    );
}

#[test]
fn incomplete_unsupported_or_disabled_packages_are_rejected_as_a_whole() {
    let skills = [skill("research"), skill("delivery")];
    assert_eq!(
        require_package(&plugin(), &skills[..1], &[], &[]).unwrap_err(),
        "plugin-skill-missing"
    );
    let mut unsupported = skills.clone();
    unsupported[1]
        .unsupported_requirements
        .push("unknown".into());
    assert_eq!(
        require_package(&plugin(), &unsupported, &[], &[]).unwrap_err(),
        "plugin-skill-unavailable"
    );
    let mut disabled = plugin();
    disabled.enabled = false;
    assert_eq!(
        require_package(&disabled, &skills, &[], &[]).unwrap_err(),
        "selected-plugin-disabled"
    );
}

#[test]
fn ignored_vendor_metadata_does_not_become_a_required_package_capability() {
    let mut package = plugin();
    for kind in ["manifest", "extension"] {
        package.revision.components.push(PluginComponent {
            name: "other-client".into(),
            kind: kind.into(),
            supported: false,
            diagnostic: Some("unsupported-client-metadata-ignored".into()),
        });
    }
    let skills = [skill("research"), skill("delivery")];
    assert!(require_package(&package, &skills, &[], &[]).is_ok());
    package.revision.components.push(PluginComponent {
        name: "required".into(),
        kind: "mcp".into(),
        supported: false,
        diagnostic: Some("unsupported-transport".into()),
    });
    assert_eq!(
        require_package(&package, &skills, &[], &[]).unwrap_err(),
        "plugin-requires-unsupported-capabilities"
    );
}

#[test]
fn packaged_services_require_exactly_one_binding_and_cannot_be_selected_directly() {
    let mut plugin = plugin();
    plugin.revision.connectors.push(PluginConnectorDeclaration {
        component_name: "sources".into(),
        name: "Sources".into(),
        description: "Evidence".into(),
        configuration: json!({"type":"streamable-http","url":"https://test/mcp"}),
        authorization: ConnectorAuthorization::None,
    });
    let connector = ConnectorDefinition {
        id: "service".into(),
        revision: "one".into(),
        name: "Sources".into(),
        description: "Evidence".into(),
        plugin_id: Some(plugin.id.clone()),
        component_name: Some("sources".into()),
        endpoint: Some(McpEndpoint::StreamableHttp {
            url: "https://test/mcp".into(),
            headers: Default::default(),
            allow_local_network: false,
        }),
        authorization: ConnectorAuthorization::None,
    };
    let connection = McpConnection {
        id: "binding".into(),
        connector_id: connector.id.clone(),
        revision: "one".into(),
        name: "Sources".into(),
        enabled: true,
        tools: vec![],
    };
    let skills = [skill("research"), skill("delivery")];
    assert_eq!(
        require_package(&plugin, &skills, std::slice::from_ref(&connector), &[]).unwrap_err(),
        "plugin-connection-missing"
    );
    assert_eq!(
        require_package(
            &plugin,
            &skills,
            std::slice::from_ref(&connector),
            &[connection.clone(), connection.clone()]
        )
        .unwrap_err(),
        "plugin-connection-binding-ambiguous"
    );
    let (resolved, _) = resolve(
        &selection(),
        &[plugin.clone()],
        &skills,
        std::slice::from_ref(&connector),
        std::slice::from_ref(&connection),
    )
    .unwrap();
    assert_eq!(resolved.connections[0].connection_id, connection.id);
    let mut direct = selection();
    direct.plugin_ids.clear();
    direct.connections = resolved.connections;
    assert_eq!(
        resolve(&direct, &[plugin], &skills, &[connector], &[connection]).unwrap_err(),
        "plugin-components-require-package-selection"
    );
}

#[test]
fn package_expansion_preserves_authorization_and_rejects_missing_required_tools() {
    let (mut connection, mut catalog, _) = super::tools::fixture();
    let connector = ConnectorDefinition {
        id: connection.connector_id.clone(),
        revision: "one".into(),
        name: "Sources".into(),
        description: "Sources".into(),
        plugin_id: Some("package".into()),
        component_name: Some("sources".into()),
        endpoint: None,
        authorization: ConnectorAuthorization::None,
    };
    let mut optional = catalog.tools[0].clone();
    optional.name = "optional-delete".into();
    connection.tools.push(McpToolPermission {
        name: optional.name.clone(),
        schema_digest: optional.schema_digest.clone(),
        read_only: false,
        policy: McpToolPolicy::Deny,
    });
    catalog.tools.push(optional);
    let tools = super::super::tools::project(
        &connection,
        catalog.clone(),
        super::super::tools::CatalogSelection::Package,
    )
    .unwrap();
    assert_eq!(tools.len(), 3);
    assert!(tools
        .iter()
        .all(|tool| tool.policy == McpToolPolicy::Confirm));
    assert_eq!(tools[0].policy, McpToolPolicy::Confirm);
    let mut required = skill("research");
    required.included_tools.push("sources/publish".into());
    required
        .included_tools
        .push("sources/resources/read".into());
    required.included_tools.push("sources/prompts/get".into());
    let mut snapshot = AgentExtensionSnapshot {
        plugins: vec![plugin()],
        id: "snapshot".into(),
        digest: "one".into(),
        skills: vec![required],
        files: vec![],
        skill_tool_bindings: Default::default(),
        connectors: vec![connector],
        connections: vec![connection.clone()],
        accounts: vec![],
        catalogs: Default::default(),
        tools,
    };
    assert!(super::super::skill_bindings::resolve(&snapshot, &[]).is_ok());
    connection.tools[0].policy = McpToolPolicy::Deny;
    snapshot.tools = super::super::tools::project(
        &connection,
        catalog,
        super::super::tools::CatalogSelection::Package,
    )
    .unwrap();
    assert_eq!(
        super::super::skill_bindings::resolve(&snapshot, &[]).unwrap_err(),
        "skill-tool-binding-unavailable: research: sources/publish"
    );
}
