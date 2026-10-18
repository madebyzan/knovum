use std::{collections::BTreeMap, path::Path};

use pulldown_cmark::{Event, Parser, Tag};
use serde_json::Value;

use crate::documents::generation::{document::ImageSource, input::DocumentFileSource};

pub(super) fn load(
    app: &tauri::AppHandle,
    content: &Value,
    markdown: Option<&str>,
) -> Result<Vec<ImageSource>, String> {
    let mut destinations = BTreeMap::new();
    if let Some(root) = content.get("tiptapContent") {
        collect_note(root, &mut destinations, 0)?;
    }
    if let Some(markdown) = markdown {
        let referenced = Parser::new(markdown)
            .filter_map(|event| match event {
                Event::Start(Tag::Image { dest_url, .. }) => Some(dest_url.into_string()),
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>();
        destinations.retain(|key, _| referenced.contains(key));
        for destination in referenced {
            destinations
                .entry(destination.clone())
                .or_insert(destination);
        }
    }
    if destinations.len() > 2_000 {
        return Err("Document exceeds 2000 image resources".into());
    }
    Ok(destinations
        .into_iter()
        .filter_map(|(destination, path)| held(app, destination, &path))
        .collect())
}

/// The picture behind one reference, when this Space holds it.
///
/// A conversion embeds bytes, so it can only embed what was imported. A reference it cannot read
/// - a badge served over the network, a path outside the Space, a file that is gone - is not
/// held, and the document model reads such a reference as its alt text.
fn held(app: &tauri::AppHandle, destination: String, path: &str) -> Option<ImageSource> {
    let path = Path::new(path);
    let path = if path.is_absolute() {
        path.to_owned()
    } else {
        crate::spaces::assets::asset_directory(app).ok()?.join(path)
    };
    if std::fs::symlink_metadata(&path)
        .ok()?
        .file_type()
        .is_symlink()
    {
        return None;
    }
    let path = crate::spaces::assets::authorize_spaces_asset(app, &path).ok()?;
    let metadata = std::fs::metadata(&path).ok()?;
    if !metadata.is_file() || metadata.len() > crate::documents::generation::images::MAX_IMAGE_BYTES
    {
        return None;
    }
    Some(ImageSource {
        destination,
        file: DocumentFileSource {
            title: String::new(),
            source_revision: crate::file_inspection::content_revision(&path).ok()?,
            path,
        },
    })
}

fn collect_note(
    node: &Value,
    destinations: &mut BTreeMap<String, String>,
    depth: usize,
) -> Result<(), String> {
    if depth > 64 {
        return Err("Document image nesting exceeds 64 levels".into());
    }
    if node.get("type").and_then(Value::as_str) == Some("image") {
        let attrs = node
            .get("attrs")
            .ok_or("Document image attributes are missing")?;
        let destination = attrs
            .get("src")
            .and_then(Value::as_str)
            .ok_or("Document image source is missing")?;
        let path = attrs
            .get("assetPath")
            .and_then(Value::as_str)
            .ok_or("Import document images into this Space before conversion")?;
        if destinations
            .get(destination)
            .is_some_and(|existing| existing != path)
        {
            return Err("Document image identity refers to different resources".into());
        }
        destinations.insert(destination.to_owned(), path.to_owned());
    }
    if let Some(children) = node.get("content").and_then(Value::as_array) {
        for child in children {
            collect_note(child, destinations, depth + 1)?;
        }
    }
    Ok(())
}
