use super::endpoint;
use crate::extensions::contracts::{McpEndpoint, PluginConnectorConfiguration};
use serde_json::json;

#[test]
fn stdio_declaration_is_installable_without_resolving_or_starting_a_program() {
    let root = std::path::Path::new("/plugin/revision");
    let data = std::path::Path::new("/plugin/data");
    let declaration = json!({"type":"stdio","command":"node","args":["${PLUGIN_ROOT}/server.js"],"cwd":"${PLUGIN_DATA}"});
    assert!(endpoint(root, data, &declaration, None).unwrap().is_none());
    let configuration = PluginConnectorConfiguration::Stdio {
        executable: "/installed/node".into(),
    };
    let resolved = endpoint(root, data, &declaration, Some(&configuration))
        .unwrap()
        .unwrap();
    let McpEndpoint::Stdio {
        executable,
        args,
        cwd,
        ..
    } = resolved
    else {
        panic!("expected stdio")
    };
    assert_eq!(executable, "/installed/node");
    assert_eq!(args, ["/plugin/revision/server.js"]);
    assert_eq!(cwd.as_deref(), Some("/plugin/data"));
}

#[test]
fn local_network_permission_requires_explicit_configuration_and_matching_transport() {
    let root = std::path::Path::new("/plugin");
    let declaration = json!({"type":"streamable-http","url":"http://localhost:8000/mcp"});
    let endpoint_without_permission = endpoint(root, root, &declaration, None).unwrap().unwrap();
    assert!(matches!(
        endpoint_without_permission,
        McpEndpoint::StreamableHttp {
            allow_local_network: false,
            ..
        }
    ));
    let permission = PluginConnectorConfiguration::StreamableHttp {
        allow_local_network: true,
    };
    assert!(matches!(
        endpoint(root, root, &declaration, Some(&permission))
            .unwrap()
            .unwrap(),
        McpEndpoint::StreamableHttp {
            allow_local_network: true,
            ..
        }
    ));
    let wrong = PluginConnectorConfiguration::Stdio {
        executable: "/installed/server".into(),
    };
    assert!(endpoint(root, root, &declaration, Some(&wrong)).is_err());
}
