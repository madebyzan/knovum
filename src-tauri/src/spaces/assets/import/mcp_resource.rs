use crate::foundation::app_error::{AppError, AppResult};
use crate::{
    extensions::{contracts::McpResourceSelection, mcp::browsing},
    spaces::knowledge::contracts::ExternalContentSource,
};
use base64::Engine;
use serde::Serialize;
use tauri::{AppHandle, Manager};

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct PreparedMcpResourceImport {
    pub prepared: super::prepare::PreparedSpaceImport,
    pub external_source: ExternalContentSource,
    pub staging_id: String,
}

#[tauri::command]
pub async fn stage_mcp_resource_import(
    app: AppHandle,
    connection_id: String,
    revision: String,
    catalog_digest: String,
    selection: McpResourceSelection,
    content_digest: String,
    content_index: usize,
) -> AppResult<PreparedMcpResourceImport> {
    let read =
        browsing::read_resource(&app, &connection_id, &revision, &catalog_digest, &selection)
            .await?;
    if read.digest != content_digest {
        return Err(AppError::invalid("mcp-resource-content-changed-review-before-import"));
    }
    let content = read
        .contents
        .get(content_index)
        .ok_or("mcp-resource-content-index-invalid")?;
    let (bytes, extension) = match content {
        rmcp::model::ResourceContents::TextResourceContents {
            text, mime_type, ..
        } => {
            if text.len() > 4 * 1024 * 1024 {
                return Err(AppError::invalid("mcp-resource-text-import-exceeds-4-mib"));
            }
            (
                text.as_bytes().to_vec(),
                if mime_type.as_deref() == Some("text/markdown") {
                    "md"
                } else {
                    "txt"
                },
            )
        }
        rmcp::model::ResourceContents::BlobResourceContents {
            blob, mime_type, ..
        } => {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(blob)
                .map_err(|_| "mcp-resource-blob-invalid")?;
            let extension = match mime_type.as_deref() {
                Some("image/png") => "png",
                Some("image/jpeg") => "jpg",
                Some("image/webp") => "webp",
                Some("audio/wav") => "wav",
                Some("audio/mpeg") => "mp3",
                Some("application/pdf") => "pdf",
                _ => "bin",
            };
            (bytes, extension)
        }
        _ => return Err(AppError::invalid("mcp-resource-content-kind-not-supported")),
    };
    let staging_id = uuid::Uuid::new_v4().to_string();
    let directory = staging_root(&app)?.join(&staging_id);
    let source = directory.join(format!("mcp-resource.{extension}"));
    let cleanup = directory.clone();
    let prepared = crate::foundation::blocking::run(move || {
        std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
        std::fs::write(&source, &bytes).map_err(|error| error.to_string())?;
        super::prepare::prepare_owned_asset(
            &app,
            source,
            tokio_util::sync::CancellationToken::new(),
        )
    })
    .await;
    let prepared = match prepared {
        Ok(prepared) => prepared,
        Err(error) => {
            std::fs::remove_dir_all(cleanup)
                .map_err(|cleanup| format!("{error}; staging cleanup failed: {cleanup}"))?;
            return Err(error.into());
        }
    };
    let external_source = ExternalContentSource::McpResource {
        connection_id,
        connection_revision: revision,
        catalog_digest,
        uri: read.uri,
        content_digest,
        captured_at: crate::foundation::clock::unix_time_millis()?,
    };
    Ok(PreparedMcpResourceImport {
        prepared,
        external_source,
        staging_id,
    })
}

#[tauri::command]
pub async fn discard_mcp_resource_import(app: AppHandle, staging_id: String) -> AppResult<()> {
    let id = uuid::Uuid::parse_str(&staging_id)
        .map_err(|_| "mcp-resource-staging-id-invalid")?
        .to_string();
    let directory = staging_root(&app)?.join(id);
    if directory.exists() {
        std::fs::remove_dir_all(directory).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn staging_root(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    Ok(app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("mcp-resource-imports"))
}

pub(crate) fn discard_interrupted(app: &AppHandle) -> Result<(), String> {
    let directory = staging_root(app)?;
    if directory.exists() {
        std::fs::remove_dir_all(directory).map_err(|error| error.to_string())?;
    }
    Ok(())
}
