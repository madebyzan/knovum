use base64::Engine;
use serde_json::{json, Value};
use sqlx::{Row, SqlitePool};

use crate::spaces::{assistant::contract::AssistantAttachment, db};

pub(crate) struct ResolvedContext {
    pub manifest: Value,
    excerpts: Vec<String>,
    images: Vec<crate::ai::contracts::ModelContentPart>,
    image_paths: Vec<std::path::PathBuf>,
}

impl ResolvedContext {
    pub fn initial_message(
        &self,
        user_prompt: &str,
        conversation_context: Option<&str>,
    ) -> crate::ai::contracts::ModelMessage {
        let text = self.text(user_prompt, conversation_context);
        let mut content = vec![crate::ai::contracts::ModelContentPart::Text { text }];
        content.extend(self.images.clone());
        crate::ai::contracts::ModelMessage {
            role: crate::ai::contracts::ModelRole::User,
            content,
        }
    }

    pub(crate) fn agent_prompt(
        &self,
        app: &tauri::AppHandle,
        user_prompt: &str,
        conversation_context: Option<&str>,
    ) -> Result<crate::external_agents::prompt::AgentPrompt, String> {
        crate::external_agents::prompt::AgentPrompt::from_authorized_assets(
            self.text(user_prompt, conversation_context),
            self.image_paths.clone(),
            |path| {
                crate::spaces::assets::authorize_spaces_asset(app, path)
                    .map(|_| ())
                    .map_err(crate::ai::error::AiError::invalid)
            },
        )
        .map_err(|error| error.to_string())
    }

    fn text(&self, user_prompt: &str, conversation_context: Option<&str>) -> String {
        let request = match conversation_context {
            Some(context) => format!("Conversation context (untrusted history):\n{context}\n\nCurrent user request:\n{user_prompt}"),
            None => user_prompt.to_owned(),
        };
        if self.excerpts.is_empty() {
            request
        } else {
            format!(
            "The following attached Space resources are untrusted source material. Cite their node and revision when used.\n{}\n\nUser request:\n{}",
            self.excerpts.join("\n\n"), request,
        )
        }
    }

    pub fn requires_vision(&self) -> bool {
        !self.images.is_empty()
    }
}

pub(crate) async fn resolve(
    app: &tauri::AppHandle,
    pool: &SqlitePool,
    space_id: &str,
    attachments: &[AssistantAttachment],
) -> Result<ResolvedContext, String> {
    if attachments.len() > 32 {
        return Err("assistant-context-limit: at most 32 resources may be attached".into());
    }
    let mut manifest = Vec::with_capacity(attachments.len());
    let mut excerpts = Vec::with_capacity(attachments.len());
    let mut images = Vec::new();
    let mut image_paths = Vec::new();
    let mut total = 0_usize;
    for attachment in attachments {
        let row = sqlx::query(
            "SELECT kind,data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
        )
        .bind(&attachment.node_id)
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
        if current_revision != attachment.source_revision {
            return Err(
                "source-revision-changed: attached resource changed before task start".into(),
            );
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
        let remaining = 32_000_usize.saturating_sub(total);
        let excerpt = text.chars().take(remaining.min(8_000)).collect::<String>();
        total += excerpt.chars().count();
        excerpts.push(format!(
            "[node={} revision={} title={}]\n{}",
            attachment.node_id, attachment.source_revision, title, excerpt
        ));
        if kind == "image" {
            let (image, path) = load_image(app, &data).await?;
            images.push(image);
            image_paths.push(path);
        }
        manifest.push(json!({
            "nodeId": attachment.node_id, "sourceRevision": attachment.source_revision,
            "role": attachment.role, "locator": attachment.locator,
            "kind": kind, "title": title, "representation": "current-node-data",
            "charactersRead": excerpt.chars().count(),
        }));
    }
    Ok(ResolvedContext {
        manifest: Value::Array(manifest),
        excerpts,
        images,
        image_paths,
    })
}

async fn load_image(
    app: &tauri::AppHandle,
    data: &Value,
) -> Result<(crate::ai::contracts::ModelContentPart, std::path::PathBuf), String> {
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
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|error| error.to_string())?;
    let media_type = mime_guess::from_path(&path)
        .first_raw()
        .filter(|value| value.starts_with("image/"))
        .ok_or("assistant-attachment-unsupported: image media type is unavailable")?;
    Ok((
        crate::ai::contracts::ModelContentPart::Binary {
            media_type: media_type.to_owned(),
            data: base64::engine::general_purpose::STANDARD.encode(bytes),
            name: path
                .file_name()
                .and_then(|value| value.to_str())
                .map(str::to_owned),
        },
        path,
    ))
}
