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
    let asset_revision = node["assetRevision"]
        .as_str()
        .filter(|_| target == ContextTarget::ExternalAgent || kind == "image");
    let asset = if let Some(revision) = asset_revision {
        Some(crate::spaces::assets::node_asset::resolve(app, &node, revision).await?)
    } else {
        None
    };
    let thumbnail = if kind == "image" {
        Some(
            super::image::thumbnail(
                asset
                    .as_ref()
                    .ok_or("image-source-unavailable")?
                    .path
                    .clone(),
            )
            .await?,
        )
    } else {
        None
    };
    let image = if target == ContextTarget::Model && kind == "image" {
        Some(
            super::image::read_image_attachment(
                &asset.as_ref().ok_or("image-source-unavailable")?.path,
            )
            .await?,
        )
    } else {
        None
    };
    let agent_path = if target == ContextTarget::ExternalAgent {
        asset.map(|asset| {
            let origin = match asset.representation {
                crate::spaces::assets::node_asset::AssetRepresentation::Original => {
                    AgentPathOrigin::SpaceAsset
                }
                crate::spaces::assets::node_asset::AssetRepresentation::Rendered => {
                    AgentPathOrigin::RenderedImage
                }
            };
            (asset.path, origin)
        })
    } else {
        None
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
