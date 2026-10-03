use crate::app_error::AppError;
use std::path::PathBuf;

use serde::Serialize;
use tauri::AppHandle;

use models::{catalog::ModelKind, ModelStatus};

mod configured_path;
mod download;
pub(crate) mod models;
pub(crate) mod preparation;
#[cfg(test)]
mod tests;
pub(crate) mod tools;
mod verification;

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct LocalResourcesStatus {
    platform: &'static str,
    ffmpeg: ToolStatus,
    yt_dlp: ToolStatus,
    deno: ToolStatus,
    models: ModelStatuses,
    kokoro: models::kokoro::KokoroPackageStatus,
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
struct ToolStatus {
    available: bool,
    path: Option<String>,
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
struct ModelStatuses {
    transcription: Vec<ModelStatus>,
    image_segmentation: ModelStatus,
    text_embedding_config: ModelStatus,
    text_embedding_modules: ModelStatus,
    text_embedding_pooling_config: ModelStatus,
    text_embedding_sentence_transformers_config: ModelStatus,
    text_embedding_tokenizer: ModelStatus,
    text_embedding_tokenizer_config: ModelStatus,
    text_embedding_weights: ModelStatus,
}

#[tauri::command]
pub async fn get_status(app: AppHandle) -> Result<LocalResourcesStatus, AppError> {
    Ok((crate::blocking::run(move || status(&app)).await)?)
}

fn status(app: &AppHandle) -> Result<LocalResourcesStatus, String> {
    Ok(LocalResourcesStatus {
        platform: tools::platform_directory(),
        ffmpeg: tool_status(app, "KNOVUM_FFMPEG_PATH", tools::ffmpeg_name())?,
        yt_dlp: tool_status_from_path(tools::resolve_yt_dlp(app)?),
        deno: tool_status_from_path(tools::resolve_deno(app)?),
        models: model_statuses(app)?,
        kokoro: models::kokoro::status(app)?,
    })
}

fn model_statuses(app: &AppHandle) -> Result<ModelStatuses, String> {
    Ok(ModelStatuses {
        transcription: models::catalog::TRANSCRIPTION_MODELS
            .iter()
            .copied()
            .map(|kind| model_status(app, kind))
            .collect::<Result<Vec<_>, _>>()?,
        image_segmentation: model_status(app, ModelKind::ImageSegmentation)?,
        text_embedding_config: model_status(app, ModelKind::TextEmbeddingConfig)?,
        text_embedding_modules: model_status(app, ModelKind::TextEmbeddingModules)?,
        text_embedding_pooling_config: model_status(app, ModelKind::TextEmbeddingPoolingConfig)?,
        text_embedding_sentence_transformers_config: model_status(
            app,
            ModelKind::TextEmbeddingSentenceTransformersConfig,
        )?,
        text_embedding_tokenizer: model_status(app, ModelKind::TextEmbeddingTokenizer)?,
        text_embedding_tokenizer_config: model_status(
            app,
            ModelKind::TextEmbeddingTokenizerConfig,
        )?,
        text_embedding_weights: model_status(app, ModelKind::TextEmbeddingWeights)?,
    })
}

fn model_status(app: &AppHandle, model: ModelKind) -> Result<ModelStatus, String> {
    models::status(app, models::catalog::descriptor(model))
}

fn tool_status(app: &AppHandle, env_key: &str, name: &str) -> Result<ToolStatus, String> {
    Ok(tool_status_from_path(tools::resolve_tool(
        app, env_key, name,
    )?))
}

fn tool_status_from_path(path: Option<PathBuf>) -> ToolStatus {
    ToolStatus {
        available: path.is_some(),
        path: path.map(|value| value.to_string_lossy().into_owned()),
    }
}
