use base64::Engine;
use serde_json::{json, Value};
use sqlx::SqlitePool;

use super::{AgentPathOrigin, ContextTarget, ResolvedPart};
use crate::spaces::db;

pub(super) struct NodeReference<'a> {
    pub node_id: &'a str,
    pub source_revision: &'a str,
    pub role: &'a str,
    pub locator: &'a Option<Value>,
}

pub(super) async fn resolve(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    space_id: &str,
    reference: NodeReference<'_>,
    remaining: usize,
    target: ContextTarget,
) -> Result<ResolvedPart, String> {
    let NodeReference {
        node_id,
        source_revision,
        role,
        locator,
    } = reference;
    let mut tx = pool.begin().await.map_err(db::sql_error)?;
    let node =
        crate::spaces::queries::knowledge_projection::get_node(&mut tx, space_id, node_id).await?;
    tx.commit().await.map_err(db::sql_error)?;
    let kind = node["kind"]
        .as_str()
        .ok_or("attached resource has no kind")?
        .to_owned();
    let data = &node["data"];
    let current_revision = data
        .get("knowledgeRevision")
        .or_else(|| data.get("sourceRevision"))
        .and_then(Value::as_str)
        .ok_or("source-unavailable: attached resource has no stable revision")?;
    if current_revision != source_revision {
        return Err("source-revision-changed: attached resource changed before task start".into());
    }
    let title = data["title"]
        .as_str()
        .ok_or("attached resource has no title")?;
    let excerpt = excerpt(data, remaining)?;
    let characters_read = excerpt.chars().count();
    let (image, agent_path, thumbnail) = if kind == "image" {
        let (path, origin) = image_path(app, data).await?;
        let thumbnail = super::image_thumbnail(path.clone()).await?;
        if target == ContextTarget::Model {
            (Some(load_image(&path).await?), None, Some(thumbnail))
        } else {
            (None, Some((path, origin)), Some(thumbnail))
        }
    } else {
        (None, None, None)
    };
    Ok(ResolvedPart {
        manifest: json!({
            "source": "node", "nodeId": node_id, "sourceRevision": source_revision,
            "role": role, "locator": locator, "kind": kind.clone(), "title": title,
            "representation": "current-node-data", "charactersRead": characters_read,
        }),
        summary: crate::spaces::assistant::AssistantMessageAttachment::Node {
            title: title.to_owned(),
            node_id: node_id.to_owned(),
            kind,
            thumbnail,
        },
        excerpt: Some(format!(
            "[node={node_id} revision={source_revision} title={title}]\n{excerpt}"
        )),
        image,
        agent_path,
        characters_read,
    })
}

fn excerpt(data: &Value, remaining: usize) -> Result<String, String> {
    let mut chunks = crate::spaces::knowledge::indexing::node::derived_chunks(data)?;
    chunks.extend(crate::spaces::knowledge::indexing::media::chunks(data)?);
    Ok(chunks
        .iter()
        .map(|chunk| chunk.text.as_str())
        .collect::<Vec<_>>()
        .join("\n\n")
        .chars()
        .take(remaining.min(8_000))
        .collect())
}

async fn image_path(
    app: &tauri::AppHandle,
    data: &Value,
) -> Result<(std::path::PathBuf, AgentPathOrigin), String> {
    let (path, origin) = if data.get("imageEdit").is_some_and(|edit| !edit.is_null()) {
        (
            crate::spaces::assets::image::rendered_preview(app, data).await?,
            AgentPathOrigin::RenderedImage,
        )
    } else {
        let source = data
            .get("assetPath")
            .and_then(Value::as_str)
            .ok_or("assistant-attachment-invalid: image has no asset path")?;
        (
            crate::spaces::assets::authorize_spaces_asset(app, std::path::Path::new(source))?,
            AgentPathOrigin::SpaceAsset,
        )
    };
    let metadata = tokio::fs::metadata(&path)
        .await
        .map_err(|error| error.to_string())?;
    if metadata.len() > 10 * 1024 * 1024 {
        return Err("assistant-attachment-too-large: image exceeds 10 MiB".into());
    }
    Ok((path, origin))
}

async fn load_image(
    path: &std::path::Path,
) -> Result<crate::ai::contracts::ModelContentPart, String> {
    let bytes = tokio::fs::read(path)
        .await
        .map_err(|error| error.to_string())?;
    let media_type = mime_guess::from_path(path)
        .first_raw()
        .filter(|value| value.starts_with("image/"))
        .ok_or("assistant-attachment-unsupported: image media type is unavailable")?;
    Ok(crate::ai::contracts::ModelContentPart::Binary {
        media_type: media_type.to_owned(),
        data: base64::engine::general_purpose::STANDARD.encode(bytes),
        name: path
            .file_name()
            .and_then(|value| value.to_str())
            .map(str::to_owned),
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn attached_notes_use_canonical_editor_content_and_respect_the_read_budget() {
        let data = serde_json::json!({
            "kind":"note", "knowledgeRevision":"node1:test", "plainText":"画布里的资料",
            "tiptapContent":{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"画布里的资料"}]}]},
        });
        assert_eq!(super::excerpt(&data, 100).unwrap(), "画布里的资料");
        assert_eq!(super::excerpt(&data, 2).unwrap(), "画布");
    }
}
