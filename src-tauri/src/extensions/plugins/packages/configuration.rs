use super::super::super::{
    content::PackageFiles, contracts::PluginComponent, mcp::endpoint::secret_name,
};
use super::manifest::diagnostic;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn read(
    files: &PackageFiles,
    components: &mut Vec<PluginComponent>,
) -> BTreeMap<String, Value> {
    let mut servers = BTreeMap::new();
    if !files.contains_key("mcp.json") {
        return servers;
    }
    let result = (|| {
        let value = super::json(files, "mcp.json")?;
        let object = value
            .as_object()
            .ok_or("mcp-configuration-must-be-object")?;
        if object.len() != 2
            || value["$schema"] != "https://agent-plugins.org/schemas/1.0.0/mcp.schema.json"
        {
            return Err("mcp-configuration-schema-or-fields-invalid".into());
        }
        let entries = value["mcpServers"]
            .as_object()
            .ok_or("mcp-server-map-required")?;
        let mut schema: Value = serde_json::from_str(include_str!("schemas/mcp.schema.json"))
            .expect("pinned MCP schema");
        schema
            .as_object_mut()
            .expect("owned schema")
            .insert("$ref".into(), json!("#/$defs/server"));
        for field in ["type", "properties", "required", "additionalProperties"] {
            schema.as_object_mut().expect("owned schema").remove(field);
        }
        let validator = jsonschema::validator_for(&schema).map_err(|error| error.to_string())?;
        for (name, server) in entries {
            let validation = validator
                .validate(server)
                .map_err(|error| error.to_string())
                .and_then(|_| validate(server));
            if let Err(error) = validation {
                diagnostic(components, name, "mcp", error);
                continue;
            }
            if server["type"] == "sse" {
                diagnostic(components, name, "mcp", "legacy-http-sse-not-supported");
                continue;
            }
            servers.insert(name.clone(), server.clone());
            components.push(PluginComponent {
                name: name.clone(),
                kind: "mcp".into(),
                supported: true,
                diagnostic: (server["type"] == "stdio").then(|| {
                    "requires-explicit-installed-executable-and-process-authorization".into()
                }),
            });
        }
        Ok::<_, String>(())
    })();
    if let Err(error) = result {
        diagnostic(components, "mcp.json", "mcp", error);
    }
    servers
}

fn validate(server: &Value) -> Result<(), String> {
    if server["type"] == "stdio" {
        let command = server["command"].as_str().expect("schema validated");
        if command.contains("${")
            || (command.contains('/') || command.contains('\\')) && !command.starts_with("./")
        {
            return Err("plugin-command-must-be-bare-token-or-plugin-relative-path".into());
        }
        if let Some(relative) = command.strip_prefix("./") {
            super::super::super::content::validate_path(relative)?;
        }
        if let Some(env) = server["env"].as_object() {
            if env.keys().any(|name| {
                secret_name(name)
                    || name.eq_ignore_ascii_case("PLUGIN_ROOT")
                    || name.eq_ignore_ascii_case("PLUGIN_DATA")
            }) {
                return Err(
                    "plugin-secrets-and-reserved-environment-must-be-client-managed".into(),
                );
            }
        }
    } else {
        let url = url::Url::parse(server["url"].as_str().expect("schema validated"))
            .map_err(|error| error.to_string())?;
        let local = url.host_str() == Some("localhost")
            || url
                .host_str()
                .and_then(|host| {
                    host.trim_matches(['[', ']'])
                        .parse::<std::net::IpAddr>()
                        .ok()
                })
                .is_some_and(|ip| ip.is_loopback());
        if !url.username().is_empty()
            || url.password().is_some()
            || url.fragment().is_some()
            || url.query_pairs().any(|(name, _)| secret_name(&name))
            || (url.scheme() != "https" && !(url.scheme() == "http" && local))
        {
            return Err("plugin-mcp-url-invalid".into());
        }
        let mut names = BTreeSet::new();
        if let Some(headers) = server["headers"].as_object() {
            for (name, value) in headers {
                if !names.insert(name.to_ascii_lowercase())
                    || secret_name(name)
                    || super::super::super::mcp::endpoint::reserved_header(name)
                {
                    return Err("plugin-secret-or-duplicate-header".into());
                }
                reqwest::header::HeaderName::from_bytes(name.as_bytes())
                    .map_err(|_| "plugin-header-name-invalid")?;
                reqwest::header::HeaderValue::from_str(value.as_str().expect("schema validated"))
                    .map_err(|_| "plugin-header-value-invalid")?;
            }
        }
    }
    Ok(())
}
