use super::super::{
    content,
    contracts::{ConnectorDefinition, McpEndpoint},
    database,
};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use tauri::AppHandle;

pub(super) fn endpoint(
    app: &AppHandle,
    id: &str,
    digest: &str,
    server: &Value,
) -> Result<McpEndpoint, String> {
    let strings = |value: &Value| -> Result<BTreeMap<String, String>, String> {
        if value.is_null() {
            Ok(Default::default())
        } else {
            serde_json::from_value(value.clone()).map_err(|error| error.to_string())
        }
    };
    if server["type"] == "streamable-http" {
        return Ok(McpEndpoint::StreamableHttp {
            url: server["url"].as_str().expect("reviewed URL").into(),
            headers: strings(&server["headers"])?,
            allow_local_network: false,
        });
    }
    let (root, data) = paths(app, id, digest)?;
    let expand = |value: &str| expand(value, &root, &data);
    let command = server["command"].as_str().expect("reviewed command");
    let executable = command
        .strip_prefix("./")
        .map(|relative| root.join(relative).to_string_lossy().into_owned())
        .unwrap_or_else(|| command.into());
    let args = server["args"]
        .as_array()
        .map(|args| {
            args.iter()
                .map(|arg| expand(arg.as_str().expect("reviewed argument")))
                .collect()
        })
        .unwrap_or_default();
    let environment = strings(&server["env"])?
        .into_iter()
        .map(|(name, value)| (name, expand(&value)))
        .collect();
    let cwd = server["cwd"].as_str().unwrap_or("${PLUGIN_ROOT}");
    let cwd = if let Some(relative) = cwd.strip_prefix("./") {
        contained(&root, relative)?
    } else if let Some(relative) = cwd.strip_prefix("${PLUGIN_ROOT}") {
        contained(&root, relative.trim_start_matches('/'))?
    } else if let Some(relative) = cwd.strip_prefix("${PLUGIN_DATA}") {
        contained(&data, relative.trim_start_matches('/'))?
    } else {
        return Err("plugin-working-directory-root-invalid".into());
    };
    Ok(McpEndpoint::Stdio {
        executable,
        args,
        cwd: Some(cwd.to_string_lossy().into_owned()),
        environment,
    })
}

pub(crate) async fn environment(
    app: &AppHandle,
    connector: &ConnectorDefinition,
) -> Result<BTreeMap<String, String>, String> {
    let Some(id) = &connector.plugin_id else {
        return Ok(BTreeMap::new());
    };
    let plugin = super::load(&database::pool(app).await?, id).await?;
    let (root, data) = paths(app, id, &plugin.revision.digest)?;
    std::fs::create_dir_all(&data).map_err(|error| error.to_string())?;
    let root = root.canonicalize().map_err(|error| error.to_string())?;
    let owned = content::root(app)?
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let data = data.canonicalize().map_err(|error| error.to_string())?;
    if !root.starts_with(&owned) || !data.starts_with(&owned) {
        return Err("plugin-content-root-escaped".into());
    }
    if let McpEndpoint::Stdio { cwd: Some(cwd), .. } = &connector.endpoint {
        let cwd = Path::new(cwd)
            .canonicalize()
            .map_err(|error| error.to_string())?;
        if !cwd.starts_with(&root) && !cwd.starts_with(&data) {
            return Err("plugin-working-directory-escaped".into());
        }
    }
    Ok([
        ("PLUGIN_ROOT".into(), root.to_string_lossy().into_owned()),
        ("PLUGIN_DATA".into(), data.to_string_lossy().into_owned()),
    ]
    .into_iter()
    .collect())
}

fn paths(app: &AppHandle, id: &str, digest: &str) -> Result<(PathBuf, PathBuf), String> {
    let root = content::root(app)?;
    Ok((
        root.join("plugins").join(id).join("revisions").join(digest),
        root.join("plugin-data").join(id),
    ))
}
fn contained(root: &Path, relative: &str) -> Result<PathBuf, String> {
    if !relative.is_empty() {
        content::validate_path(relative)?;
    }
    Ok(root.join(relative))
}
fn expand(value: &str, root: &Path, data: &Path) -> String {
    let mut result = String::new();
    let mut remaining = value;
    while let Some(offset) = remaining.find("${PLUGIN_") {
        result.push_str(&remaining[..offset]);
        remaining = &remaining[offset..];
        if let Some(rest) = remaining.strip_prefix("${PLUGIN_ROOT}") {
            result.push_str(&root.to_string_lossy());
            remaining = rest;
        } else if let Some(rest) = remaining.strip_prefix("${PLUGIN_DATA}") {
            result.push_str(&data.to_string_lossy());
            remaining = rest;
        } else {
            result.push_str("${PLUGIN_");
            remaining = &remaining[9..];
        }
    }
    result.push_str(remaining);
    result
}
