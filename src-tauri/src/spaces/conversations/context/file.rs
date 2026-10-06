use std::path::Path;

use serde_json::json;

use super::{AgentPathOrigin, ContextTarget, ResolvedPart};
use crate::spaces::conversations::upload::{self, UploadKind};

pub(super) async fn resolve(
    app: &tauri::AppHandle,
    path: &str,
    source_revision: &str,
    remaining: usize,
    target: ContextTarget,
) -> Result<ResolvedPart, String> {
    let source = upload::authorize(app, path).map_err(|error| error.to_string())?;
    let kind = upload::classify(&source).map_err(|error| error.to_string())?;
    let size = std::fs::metadata(&source)
        .map_err(|error| error.to_string())?
        .len();
    upload::enforce_size(kind, size).map_err(|error| error.to_string())?;
    verify_revision(source.clone(), source_revision.to_owned()).await?;
    let name = source
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or("conversation-upload-invalid: filename is not Unicode")?
        .to_owned();
    let (excerpt, image, agent_path, characters_read) = match (target, kind) {
        (ContextTarget::ExternalAgent, _) => (
            None,
            None,
            Some((source.clone(), AgentPathOrigin::Upload)),
            0,
        ),
        (ContextTarget::Model, UploadKind::Image(_)) => (
            None,
            Some(super::image::read_image_attachment(&source).await?),
            None,
            0,
        ),
        (ContextTarget::Model, _) => {
            if remaining == 0 {
                return Err(
                    "conversation-context-limit: attached text exceeds the context budget".into(),
                );
            }
            let file = source.clone();
            let extracted = crate::blocking::run(move || extract_text(&file, kind)).await?;
            let text: String = extracted.chars().take(remaining).collect();
            let count = text.chars().count();
            (
                Some(format!("[file={name} revision={source_revision}]\n{text}")),
                None,
                None,
                count,
            )
        }
    };
    let kind_name = match kind {
        UploadKind::Image(_) => "image",
        UploadKind::Text => "text",
        UploadKind::Pdf => "pdf",
        UploadKind::Office => "office",
    };
    let thumbnail = if matches!(kind, UploadKind::Image(_)) {
        Some(super::image::thumbnail(source.clone()).await?)
    } else {
        None
    };
    verify_revision(source, source_revision.to_owned()).await?;
    Ok(ResolvedPart {
        manifest: json!({
            "source": "file", "kind": kind_name,
            "name": name.clone(), "sourceRevision": source_revision,
            "size": size, "charactersRead": characters_read,
        }),
        summary: crate::spaces::conversations::ConversationMessageAttachment::File {
            title: name,
            kind: kind_name.to_owned(),
            thumbnail,
        },
        excerpt,
        image,
        agent_path,
        characters_read,
    })
}

async fn verify_revision(source: std::path::PathBuf, revision: String) -> Result<(), String> {
    crate::blocking::run(move || {
        crate::file_inspection::verify_content_revision(&source, &revision)
    })
    .await
}

fn extract_text(path: &Path, kind: UploadKind) -> Result<String, String> {
    let cancellation = tokio_util::sync::CancellationToken::new();
    let text = match kind {
        UploadKind::Text => String::from_utf8(
            upload::read_bounded(path, 20 * 1024 * 1024).map_err(|error| error.to_string())?,
        )
        .map_err(|error| format!("conversation-upload-invalid: text is not UTF-8: {error}"))?,
        UploadKind::Pdf => crate::documents::extraction::pdf::extract(path, &cancellation)?
            .extracted_text
            .ok_or("conversation-upload-empty: PDF has no extractable text")?,
        UploadKind::Office => crate::documents::extraction::office::extract(path, &cancellation)?
            .extracted_text
            .ok_or("conversation-upload-empty: document has no extractable text")?,
        UploadKind::Image(_) => {
            return Err("conversation-upload-invalid: image cannot be read as text".into())
        }
    };
    if text.trim().is_empty() {
        return Err("conversation-upload-empty: file has no readable text".into());
    }
    Ok(text)
}
