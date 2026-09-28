use base64::Engine;
use serde_json::{json, Value};
use sqlx::{Row, SqlitePool};

use super::{ContextTarget, ResolvedPart};
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
    let row = sqlx::query(
        "SELECT kind,data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
    )
    .bind(node_id)
    .bind(space_id)
    .fetch_optional(pool)
    .await
    .map_err(db::sql_error)?
    .ok_or("source-unavailable: attached resource is unavailable")?;
    let kind: String = row.try_get("kind").map_err(db::sql_error)?;
    let data: Value =
        serde_json::from_str(&row.try_get::<String, _>("data").map_err(db::sql_error)?)
            .map_err(db::json_error)?;
    let current_revision = data
        .get("knowledgeRevision")
        .or_else(|| data.get("sourceRevision"))
        .and_then(Value::as_str)
        .ok_or("source-unavailable: attached resource has no stable revision")?;
    if current_revision != source_revision {
        return Err("source-revision-changed: attached resource changed before task start".into());
    }
    let title = data
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("Untitled");
    let text = data
        .get("text")
        .or_else(|| data.get("transcript"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let excerpt: String = text.chars().take(remaining.min(8_000)).collect();
    let characters_read = excerpt.chars().count();
    let (image, agent_path, thumbnail) = if kind == "image" {
        let path = image_path(app, &data).await?;
        let thumbnail = super::image_thumbnail(path.clone()).await?;
        if target == ContextTarget::Model {
            (Some(load_image(&path).await?), None, Some(thumbnail))
        } else {
            (None, Some(path), Some(thumbnail))
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
            title: title.to_owned(), node_id: node_id.to_owned(), kind, thumbnail,
        },
        excerpt: Some(format!(
            "[node={node_id} revision={source_revision} title={title}]\n{excerpt}"
        )),
        image,
        agent_path,
        uploaded: false,
        characters_read,
    })
}

async fn image_path(app: &tauri::AppHandle, data: &Value) -> Result<std::path::PathBuf, String> {
    let path = data
        .get("assetPath")
        .and_then(Value::as_str)
        .ok_or("assistant-attachment-invalid: image has no asset path")?;
    let path = crate::spaces::assets::authorize_spaces_asset(app, std::path::Path::new(path))?;
    let metadata = tokio::fs::metadata(&path)
        .await
        .map_err(|error| error.to_string())?;
    if metadata.len() > 10 * 1024 * 1024 {
        return Err("assistant-attachment-too-large: image exceeds 10 MiB".into());
    }
    Ok(path)
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
