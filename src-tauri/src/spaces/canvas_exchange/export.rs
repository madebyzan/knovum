use std::{
    collections::{HashMap, HashSet},
    io::Write,
    path::{Component, Path, PathBuf},
};

use serde::Deserialize;
use tauri::AppHandle;

use super::contract::{JsonCanvasDocumentV1, JsonCanvasNodeV1};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ExportResource {
    source_path: String,
    relative_path: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ExportRequest {
    destination_path: String,
    source: String,
    resources: Vec<ExportResource>,
}

pub(super) fn write(app: &AppHandle, request: ExportRequest) -> Result<(), String> {
    let destination = PathBuf::from(&request.destination_path);
    crate::spaces::assets::authorize_destination(app, &destination)?;
    if destination.extension().and_then(|part| part.to_str()) != Some("canvas") {
        return Err("JSON Canvas export requires a .canvas destination".into());
    }
    let mut document: JsonCanvasDocumentV1 =
        serde_json::from_str(&request.source).map_err(|error| error.to_string())?;
    super::validation::validate(&document)?;
    let expected: HashSet<&str> = document
        .nodes
        .iter()
        .filter_map(|node| match node {
            JsonCanvasNodeV1::File(file) => Some(file.file.as_str()),
            _ => None,
        })
        .collect();
    let parent = destination
        .parent()
        .ok_or("JSON Canvas destination has no parent")?
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let mut seen = HashSet::new();
    let mut plans = Vec::new();
    for resource in &request.resources {
        let parts: Vec<_> = Path::new(&resource.relative_path).components().collect();
        if parts.len() != 2
            || parts[0] != Component::Normal("assets".as_ref())
            || !matches!(parts[1], Component::Normal(_))
            || resource.relative_path.contains('\\')
            || !expected.contains(resource.relative_path.as_str())
            || !seen.insert(resource.relative_path.as_str())
        {
            return Err("JSON Canvas resource must be a unique assets/file reference".into());
        }
        let source =
            crate::spaces::assets::authorize_spaces_asset(app, Path::new(&resource.source_path))?;
        if !source.is_file() {
            return Err("JSON Canvas resource is not a regular file".into());
        }
        plans.push((source, resource.relative_path.clone()));
    }
    if seen != expected {
        return Err("JSON Canvas resources do not match its file nodes".into());
    }
    publish(&destination, &parent, &mut document, &plans)
}

fn publish(
    destination: &Path,
    parent: &Path,
    document: &mut JsonCanvasDocumentV1,
    plans: &[(PathBuf, String)],
) -> Result<(), String> {
    // Each publication owns immutable resources; an existing export is never changed
    // until its complete replacement document is ready.
    let identity = uuid::Uuid::new_v4();
    let directory_name = format!("canvas-assets-{identity}");
    let directory = parent.join(&directory_name);
    let temporary = parent.join(format!(".canvas-export-{identity}"));
    let mut owns_directory = false;
    let mut owns_temporary = false;
    let result = (|| {
        let mut references = HashMap::new();
        if !plans.is_empty() {
            std::fs::create_dir(&directory).map_err(|error| error.to_string())?;
            owns_directory = true;
        }
        for (source, reference) in plans {
            let filename = Path::new(reference)
                .file_name()
                .ok_or("Resource filename is unavailable")?;
            let target = directory.join(filename);
            let mut input = std::fs::File::open(source).map_err(|error| error.to_string())?;
            let mut output = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(target)
                .map_err(|error| error.to_string())?;
            std::io::copy(&mut input, &mut output).map_err(|error| error.to_string())?;
            output.sync_all().map_err(|error| error.to_string())?;
            references.insert(
                reference.as_str(),
                format!(
                    "{directory_name}/{}",
                    filename.to_str().ok_or("Resource filename is not UTF-8")?
                ),
            );
        }
        for node in &mut document.nodes {
            if let JsonCanvasNodeV1::File(file) = node {
                file.file = references
                    .get(file.file.as_str())
                    .ok_or("Resource publication reference is unavailable")?
                    .clone();
            }
        }
        let source = serde_json::to_vec(document).map_err(|error| error.to_string())?;
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|error| error.to_string())?;
        owns_temporary = true;
        output
            .write_all(&source)
            .map_err(|error| error.to_string())?;
        output.sync_all().map_err(|error| error.to_string())?;
        drop(output);
        std::fs::rename(&temporary, destination).map_err(|error| error.to_string())
    })();
    if result.is_err() {
        if owns_temporary {
            if let Err(error) = std::fs::remove_file(&temporary) {
                log::warn!("Could not remove unpublished JSON Canvas document: {error}");
            }
        }
        if owns_directory {
            if let Err(error) = std::fs::remove_dir_all(&directory) {
                log::warn!("Could not remove unpublished JSON Canvas resources: {error}");
            }
        }
    }
    result
}
