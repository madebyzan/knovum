mod configuration;
mod manifest;
use super::super::{
    content::{self, PackageFiles},
    contracts::{
        ConnectorAuthorization, PluginComponent, PluginConnectorDeclaration, PluginInspection,
        SkillRevision,
    },
};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct KnovumManifestExtension {
    #[serde(default)]
    skill_requirements: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    connectors: BTreeMap<String, ConnectorDeclaration>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConnectorDeclaration {
    name: String,
    description: String,
    authorization: ConnectorAuthorization,
}

pub(super) struct ReviewedPackage {
    pub inspection: PluginInspection,
    pub skills: Vec<SkillRevision>,
}

pub(super) fn inspect(
    files: &PackageFiles,
    source_revision: String,
) -> Result<ReviewedPackage, String> {
    content::validate_files(files)?;
    let mut components = Vec::new();
    let manifest = manifest::read(files, &mut components)?;
    let extension = manifest
        .extensions
        .as_ref()
        .and_then(|extensions| extensions.get("com.zand.knovum"))
        .map(|value| serde_json::from_value::<KnovumManifestExtension>(value.clone()))
        .transpose()
        .map_err(|error| error.to_string())?
        .unwrap_or_default();
    let mut mcp_servers = configuration::read(files, &mut components);
    mcp_servers.retain(|name, server| {
        let authorization = extension
            .connectors
            .get(name)
            .map(|declaration| &declaration.authorization)
            .unwrap_or(&ConnectorAuthorization::None);
        let validation = if server["type"] == "stdio" {
            super::super::connectors::requirements::validate_stdio(authorization)
        } else {
            super::super::connectors::requirements::validate_http(authorization)
        };
        if let Err(error) = validation {
            let component = components
                .iter_mut()
                .find(|component| component.kind == "mcp" && component.name == *name)
                .expect("parsed MCP component");
            component.supported = false;
            component.diagnostic = Some(error);
            return false;
        }
        true
    });
    let mut skills = Vec::new();
    if files.contains_key("skills") {
        manifest::diagnostic(
            &mut components,
            "skills",
            "skill",
            "skills-component-must-be-directory",
        );
    }
    if files.keys().any(|path| path.starts_with("mcp.json/")) {
        manifest::diagnostic(
            &mut components,
            "mcp.json",
            "mcp",
            "mcp-component-must-be-file",
        );
    }
    for name in extension
        .connectors
        .keys()
        .filter(|name| !mcp_servers.contains_key(*name))
    {
        manifest::diagnostic(
            &mut components,
            name,
            "connector",
            "connector-mcp-component-unavailable",
        );
    }
    for name in extension
        .skill_requirements
        .keys()
        .filter(|name| !files.contains_key(&format!("skills/{name}/SKILL.md")))
    {
        manifest::diagnostic(
            &mut components,
            name,
            "skill",
            "skill-requirements-reference-missing-component",
        );
    }
    for path in files
        .keys()
        .filter(|path| path.starts_with("skills/") && path.ends_with("/SKILL.md"))
    {
        let parts = path.split('/').collect::<Vec<_>>();
        if parts.len() != 3 {
            continue;
        }
        let name = parts[1];
        let prefix = format!("skills/{name}/");
        let skill_files = files
            .iter()
            .filter_map(|(path, bytes)| {
                path.strip_prefix(&prefix)
                    .map(|relative| (relative.to_owned(), bytes.clone()))
            })
            .collect();
        match super::super::skills::review(&skill_files, name) {
            Ok(mut skill) => {
                skill.requirements = extension
                    .skill_requirements
                    .get(name)
                    .cloned()
                    .unwrap_or_default();
                let missing = skill
                    .requirements
                    .iter()
                    .filter(|requirement| !mcp_servers.contains_key(*requirement))
                    .collect::<Vec<_>>();
                let diagnostic = (!missing.is_empty())
                    .then(|| format!("skill-mcp-dependency-unavailable: {missing:?}"));
                components.push(PluginComponent {
                    name: name.into(),
                    kind: "skill".into(),
                    supported: diagnostic.is_none(),
                    diagnostic,
                });
                skills.push(skill);
            }
            Err(error) => manifest::diagnostic(&mut components, name, "skill", error),
        }
    }
    if let Some(extensions) = &manifest.extensions {
        for name in extensions
            .keys()
            .filter(|name| name.as_str() != "com.zand.knovum")
        {
            manifest::diagnostic(
                &mut components,
                name,
                "extension",
                "client-extension-not-supported",
            );
        }
    }
    for path in files.keys() {
        if let Some((name, _)) = path.split_once('/') {
            if name.contains('.')
                && name != "com.zand.knovum"
                && !components
                    .iter()
                    .any(|component| component.kind == "extension" && component.name == name)
            {
                manifest::diagnostic(
                    &mut components,
                    name,
                    "extension",
                    "client-extension-directory-not-supported",
                );
            }
        }
    }
    let mut connectors = Vec::new();
    for (name, configuration) in mcp_servers {
        let declaration = extension.connectors.get(&name);
        let authorization = declaration
            .map(|declaration| declaration.authorization.clone())
            .unwrap_or(ConnectorAuthorization::None);
        let display_name = declaration
            .map(|declaration| declaration.name.clone())
            .unwrap_or_else(|| name.clone());
        if display_name.trim().is_empty() {
            return Err("connector-display-name-required".into());
        }
        let description = declaration
            .map(|declaration| declaration.description.clone())
            .or_else(|| manifest.description.clone())
            .unwrap_or_default();
        connectors.push(PluginConnectorDeclaration {
            component_name: name,
            name: display_name,
            description,
            configuration,
            authorization,
        });
    }
    Ok(ReviewedPackage {
        inspection: PluginInspection {
            manifest,
            digest: content::files_digest(files),
            source_revision,
            components,
            connectors,
            file_count: files.len() as u32,
            byte_length: files.values().map(Vec::len).sum::<usize>() as u32,
        },
        skills,
    })
}

fn json(files: &PackageFiles, path: &str) -> Result<Value, String> {
    serde_json::from_slice(
        files
            .get(path)
            .ok_or_else(|| format!("plugin-file-required: {path}"))?,
    )
    .map_err(|error| format!("{path}: {error}"))
}
fn validate_schema(value: &Value, text: &str, filename: &str) -> Result<(), String> {
    let schema: Value = serde_json::from_str(text).expect("pinned standard schema");
    jsonschema::validator_for(&schema)
        .map_err(|error| error.to_string())?
        .validate(value)
        .map_err(|error| format!("plugin-schema-invalid: {filename}: {error}"))
}

#[cfg(test)]
mod tests;
