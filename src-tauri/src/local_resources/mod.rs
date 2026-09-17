use std::path::PathBuf;

use serde::Serialize;
use tauri::{plugin::TauriPlugin, AppHandle, Wry};

use models::{catalog::ModelKind, ModelStatus};

pub(crate) mod models;
pub(crate) mod tools;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalResourcesStatus {
    platform: &'static str,
    ffmpeg: ToolStatus,
    yt_dlp: ToolStatus,
    deno: ToolStatus,
    whisper_cli: ToolStatus,
    models: ModelStatuses,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ToolStatus {
    available: bool,
    path: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelStatuses {
    transcription: ModelStatus,
    transcription_vad: ModelStatus,
    text_embedding_config: ModelStatus,
    text_embedding_tokenizer: ModelStatus,
    text_embedding_weights: ModelStatus,
}

#[tauri::command]
pub async fn get_status(app: AppHandle) -> Result<LocalResourcesStatus, String> {
    crate::blocking::run(move || status(&app)).await
}

#[tauri::command]
pub async fn remove_model(app: AppHandle, model: ModelKind) -> Result<(), String> {
    if matches!(
        model,
        ModelKind::TextEmbeddingConfig
            | ModelKind::TextEmbeddingTokenizer
            | ModelKind::TextEmbeddingWeights
    ) {
        crate::ai::capabilities::embedding::unload()?;
    }
    crate::blocking::run(move || models::uninstall(&app, models::catalog::descriptor(model))).await
}

fn status(app: &AppHandle) -> Result<LocalResourcesStatus, String> {
    Ok(LocalResourcesStatus {
        platform: tools::platform_directory(),
        ffmpeg: tool_status(app, "KNOVUM_FFMPEG_PATH", tools::ffmpeg_name()),
        yt_dlp: tool_status(app, "KNOVUM_YT_DLP_PATH", tools::yt_dlp_name()),
        deno: tool_status(app, "KNOVUM_DENO_PATH", tools::deno_name()),
        whisper_cli: tool_status(app, "KNOVUM_WHISPER_PATH", tools::whisper_cli_name()),
        models: model_statuses(app)?,
    })
}

fn model_statuses(app: &AppHandle) -> Result<ModelStatuses, String> {
    Ok(ModelStatuses {
        transcription: model_status(app, ModelKind::Transcription)?,
        transcription_vad: model_status(app, ModelKind::TranscriptionVad)?,
        text_embedding_config: model_status(app, ModelKind::TextEmbeddingConfig)?,
        text_embedding_tokenizer: model_status(app, ModelKind::TextEmbeddingTokenizer)?,
        text_embedding_weights: model_status(app, ModelKind::TextEmbeddingWeights)?,
    })
}

fn model_status(app: &AppHandle, model: ModelKind) -> Result<ModelStatus, String> {
    models::status(app, models::catalog::descriptor(model))
}

fn tool_status(app: &AppHandle, env_key: &str, name: &str) -> ToolStatus {
    tool_status_from_path(tools::resolve_tool(app, env_key, name))
}

fn tool_status_from_path(path: Option<PathBuf>) -> ToolStatus {
    ToolStatus {
        available: path.is_some(),
        path: path.map(|value| value.to_string_lossy().into_owned()),
    }
}

pub fn plugin() -> TauriPlugin<Wry> {
    tauri::plugin::Builder::new("local-resources")
        .invoke_handler(tauri::generate_handler![
            get_status,
            crate::local_resources::models::commands::install_models,
            crate::local_resources::models::commands::cancel_model_install,
            crate::local_resources::models::commands::list_model_installs,
            remove_model,
        ])
        .build()
}
