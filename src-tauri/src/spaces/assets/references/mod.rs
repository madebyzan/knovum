mod asset_url;
mod markdown;

use std::collections::BTreeSet;

use serde_json::Value;

const TOP_LEVEL_ASSET_FIELDS: [&str; 3] = ["assetPath", "faviconPath", "previewImagePath"];
pub(crate) const PACKAGE_ASSET_PREFIX: &str = "knovum-package-asset:";

/// Where an asset reference lives in node data.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum AssetReference {
    /// The node's own managed asset; its digest is what the node's `sourceRevision` records.
    Source,
    /// A canonical reference to another packaged asset.
    Embedded,
    /// A display attribute that mirrors the canonical one. It holds a package token once packaged,
    /// but may hold a remote URL while the node is stored.
    Display,
}

/// The single traversal of every asset-bearing location in node data. Packaging rewrites exactly
/// these locations into asset tokens, so both the stored-path collector and the package-token
/// collector are expressed through it and cannot drift apart.
pub(crate) fn visit_asset_references(value: &Value, visit: &mut impl FnMut(AssetReference, &str)) {
    let Some(object) = value.as_object() else {
        return;
    };
    for field in TOP_LEVEL_ASSET_FIELDS {
        if let Some(path) = object.get(field).and_then(Value::as_str) {
            let kind = if field == "assetPath" {
                AssetReference::Source
            } else {
                AssetReference::Embedded
            };
            visit(kind, path);
        }
    }
    if object.contains_key("origin") {
        if let Some(path) = object.get("previewPath").and_then(Value::as_str) {
            visit(AssetReference::Embedded, path);
        }
    }
    if let Some(content) = object.get("tiptapContent") {
        visit_tiptap_references(content, visit);
    }
    if let Some(resources) = object
        .get("documentCoverage")
        .and_then(|value| value.get("resources"))
        .and_then(Value::as_array)
    {
        for resource in resources {
            if let Some(path) = resource.get("path").and_then(Value::as_str) {
                visit(AssetReference::Embedded, path);
            }
        }
    }
    if let Some(text) = object.get("text").and_then(Value::as_str) {
        markdown::visit_references(text, visit);
    }
}

pub(crate) fn collect_node_asset_paths(value: &Value, paths: &mut BTreeSet<String>) {
    visit_asset_references(value, &mut |kind, path| {
        if kind != AssetReference::Display {
            insert_path(paths, path);
        }
    });
}

pub(crate) fn rewrite_node_asset_paths<F>(value: &mut Value, mut rewrite: F)
where
    F: FnMut(&str) -> String,
{
    let Some(object) = value.as_object_mut() else {
        return;
    };
    for field in TOP_LEVEL_ASSET_FIELDS {
        if let Some(path) = object.get(field).and_then(Value::as_str).map(str::to_owned) {
            object.insert(field.to_owned(), Value::String(rewrite(&path)));
        }
    }
    if object.contains_key("origin") {
        if let Some(path) = object
            .get("previewPath")
            .and_then(Value::as_str)
            .map(str::to_owned)
        {
            object.insert("previewPath".into(), Value::String(rewrite(&path)));
        }
    } else {
        object.remove("previewPath");
    }
    if let Some(content) = object.get_mut("tiptapContent") {
        rewrite_tiptap_asset_paths(content, &mut rewrite);
    }
    if let Some(resources) = object
        .get_mut("documentCoverage")
        .and_then(|value| value.get_mut("resources"))
        .and_then(Value::as_array_mut)
    {
        for resource in resources {
            if let Some(path) = resource
                .get("path")
                .and_then(Value::as_str)
                .map(str::to_owned)
            {
                resource["path"] = Value::String(rewrite(&path));
            }
        }
    }
    if let Some(text) = object
        .get("text")
        .and_then(Value::as_str)
        .map(str::to_owned)
    {
        let rewritten = markdown::rewrite_asset_paths(&text, &mut rewrite);
        object.insert("text".to_owned(), Value::String(rewritten));
    }
}

fn visit_tiptap_references(value: &Value, visit: &mut impl FnMut(AssetReference, &str)) {
    let Some(object) = value.as_object() else {
        return;
    };
    if object.get("type").and_then(Value::as_str) == Some("image") {
        if let Some(attrs) = object.get("attrs").and_then(Value::as_object) {
            if let Some(path) = attrs.get("assetPath").and_then(Value::as_str) {
                visit(AssetReference::Embedded, path);
            }
            if let Some(src) = attrs.get("src").and_then(Value::as_str) {
                visit(AssetReference::Display, src);
            }
        }
    }
    if let Some(content) = object.get("content").and_then(Value::as_array) {
        for child in content {
            visit_tiptap_references(child, visit);
        }
    }
}

fn rewrite_tiptap_asset_paths<F>(value: &mut Value, rewrite: &mut F)
where
    F: FnMut(&str) -> String,
{
    let Some(object) = value.as_object_mut() else {
        return;
    };
    if object.get("type").and_then(Value::as_str) == Some("image") {
        if let Some(attrs) = object.get_mut("attrs").and_then(Value::as_object_mut) {
            if let Some(path) = image_asset_path(attrs) {
                let replacement = rewrite(&path);
                attrs.insert("assetPath".to_owned(), Value::String(replacement.clone()));
                attrs.insert("src".to_owned(), Value::String(replacement));
            }
        }
    }
    if let Some(content) = object.get_mut("content").and_then(Value::as_array_mut) {
        for child in content {
            rewrite_tiptap_asset_paths(child, rewrite);
        }
    }
}

fn image_asset_path(attrs: &serde_json::Map<String, Value>) -> Option<String> {
    attrs
        .get("assetPath")
        .and_then(Value::as_str)
        .filter(|path| !path.is_empty() && !path.starts_with(PACKAGE_ASSET_PREFIX))
        .map(str::to_owned)
}

fn insert_path(paths: &mut BTreeSet<String>, path: &str) {
    if !path.is_empty() && !path.starts_with(PACKAGE_ASSET_PREFIX) {
        paths.insert(path.to_owned());
    }
}

#[cfg(test)]
mod tests;
