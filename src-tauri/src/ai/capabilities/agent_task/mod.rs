mod inputs;
mod output;
mod workspace;

use std::sync::{Arc, Mutex};

use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::{AgentKind, GeneratedImage, GeneratedVideo},
    error::{AiError, AiErrorCode, AiResult},
    providers::agents::{prompt::AgentPrompt, session, session::AgentSessionUpdate},
};

pub async fn analyze_image(
    app: &AppHandle,
    kind: AgentKind,
    system_prompt: &str,
    instruction: &str,
    image_path: &str,
    max_tokens: Option<u32>,
    cancellation: CancellationToken,
) -> AiResult<String> {
    crate::ai::settings::ensure_agent_enabled(app, kind)?;
    let limit = max_tokens
        .map(|value| format!(" Keep the response within approximately {value} tokens."))
        .unwrap_or_default();
    let prompt = format!(
        "{system_prompt}\n\n{instruction}{limit}\nAnalyze only the attached image and return the requested response directly."
    );
    let workspace = workspace::TaskWorkspace::new(app)?;
    let attachments = inputs::materialize(&workspace, vec![image_path.to_owned()]).await?;
    let text = execute(kind, &workspace, prompt, attachments, cancellation).await?;
    let text = super::response_text::strip_markdown_fences(&text);
    if text.is_empty() {
        return Err(AiError::new(
            AiErrorCode::EmptyResponse,
            "Desktop Agent returned no visual analysis",
        ));
    }
    Ok(text.to_owned())
}

pub async fn generate_image(
    app: &AppHandle,
    kind: AgentKind,
    prompt: &str,
    aspect_ratio: &str,
    cancellation: CancellationToken,
) -> AiResult<GeneratedImage> {
    crate::ai::settings::ensure_agent_enabled(app, kind)?;
    let workspace = workspace::TaskWorkspace::new(app)?;
    let task = format!(
        "Create one finished image for this request:\n{prompt}\n\nRequired aspect ratio: {aspect_ratio}. Use your installed skills or MCP tools as needed. Save exactly one final PNG, JPEG, or WebP image in the current workspace. Remove intermediate media files before finishing."
    );
    execute(kind, &workspace, task, Vec::new(), cancellation).await?;
    let bytes = output::read_unique(&workspace, output::ArtifactType::Image).await?;
    Ok(GeneratedImage {
        bytes,
        provider: kind.as_str().to_owned(),
        model: crate::ai::providers::agents::registry::descriptor(kind)
            .display_name
            .to_owned(),
        revised_prompt: None,
    })
}

pub async fn generate_video(
    app: &AppHandle,
    kind: AgentKind,
    prompt: &str,
    ratio: &str,
    duration: u8,
    references: Vec<String>,
    cancellation: CancellationToken,
) -> AiResult<GeneratedVideo> {
    crate::ai::settings::ensure_agent_enabled(app, kind)?;
    let workspace = workspace::TaskWorkspace::new(app)?;
    let attachments = inputs::materialize(&workspace, references).await?;
    let task = format!(
        "Create one finished video for this request:\n{prompt}\n\nRequired aspect ratio: {ratio}. Required duration: {duration} seconds. Treat attached images as visual references. Use your installed skills or MCP tools as needed. Save exactly one final MP4 video in the current workspace. Remove intermediate media files before finishing."
    );
    execute(kind, &workspace, task, attachments, cancellation).await?;
    let bytes = output::read_unique(&workspace, output::ArtifactType::Mp4Video).await?;
    Ok(GeneratedVideo {
        bytes,
        provider: kind.as_str().to_owned(),
        model: crate::ai::providers::agents::registry::descriptor(kind)
            .display_name
            .to_owned(),
        operation_id: uuid::Uuid::new_v4().to_string(),
    })
}

async fn execute(
    kind: AgentKind,
    workspace: &workspace::TaskWorkspace,
    text: String,
    attachments: Vec<std::path::PathBuf>,
    cancellation: CancellationToken,
) -> AiResult<String> {
    let messages = Arc::new(Mutex::new(String::new()));
    let collected = Arc::clone(&messages);
    let on_update = Arc::new(move |update| {
        if let AgentSessionUpdate::Message(text) = update {
            let mut current = collected.lock().map_err(|_| {
                AiError::internal("Desktop Agent response collector is unavailable")
            })?;
            current.push_str(&text);
        }
        Ok(())
    });
    let prompt = AgentPrompt::from_internal(text, attachments)?;
    session::run(
        kind,
        workspace.path().to_path_buf(),
        prompt,
        cancellation,
        on_update,
        Vec::new(),
    )
    .await?;
    let response = messages
        .lock()
        .map(|value| value.trim().to_owned())
        .map_err(|_| AiError::internal("Desktop Agent response collector is unavailable"))?;
    Ok(response)
}
