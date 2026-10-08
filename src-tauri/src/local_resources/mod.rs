use crate::app_error::AppError;
use std::path::PathBuf;

use serde::Serialize;
use tauri::AppHandle;

use models::{catalog::ModelKind, ModelStatus};

mod download;
mod filesystem;
pub(crate) mod model_catalog;
pub(crate) mod model_commands;
mod model_metadata;
pub(crate) mod model_selection;
pub(crate) mod models;
pub(crate) mod preparation;
pub(crate) mod runtimes;
pub(crate) mod system_memory;
pub(crate) mod tools;
mod verification;

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct LocalResourcesStatus {
    platform: &'static str,
    ffmpeg: ToolStatus,
    yt_dlp: ToolStatus,
    deno: ToolStatus,
    python_runtimes: PythonRuntimeStatuses,
    models: ModelStatuses,
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "kebab-case")]
struct PythonRuntimeStatuses {
    resources: runtimes::python::RuntimeStatus,
    light_inference: runtimes::python::RuntimeStatus,
    torch_inference: runtimes::python::RuntimeStatus,
    voice_generation: runtimes::python::RuntimeStatus,
    agent: runtimes::python::RuntimeStatus,
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
    voxcpm2: Vec<ModelStatus>,
    speech_alignment: Vec<ModelStatus>,
    speech_separation: Vec<ModelStatus>,
    diarization: Vec<ModelStatus>,
}

#[tauri::command]
pub async fn get_status(app: AppHandle) -> Result<LocalResourcesStatus, AppError> {
    Ok(crate::blocking::run(move || status(&app)).await?)
}

fn status(app: &AppHandle) -> Result<LocalResourcesStatus, String> {
    Ok(LocalResourcesStatus {
        platform: tools::platform_directory(),
        ffmpeg: tool_status_from_path(tools::resolve_ffmpeg(app)?),
        yt_dlp: tool_status_from_path(tools::resolve_yt_dlp(app)?),
        deno: tool_status_from_path(tools::resolve_deno(app)?),
        python_runtimes: PythonRuntimeStatuses {
            resources: runtimes::python::status(app, runtimes::python::RuntimeProfile::Resources),
            light_inference: runtimes::python::status(
                app,
                runtimes::python::RuntimeProfile::LightInference,
            ),
            torch_inference: runtimes::python::status(
                app,
                runtimes::python::RuntimeProfile::TorchInference,
            ),
            voice_generation: runtimes::python::status(
                app,
                runtimes::python::RuntimeProfile::VoiceGeneration,
            ),
            agent: runtimes::python::status(app, runtimes::python::RuntimeProfile::Agent),
        },
        models: model_statuses(app)?,
    })
}

fn model_statuses(app: &AppHandle) -> Result<ModelStatuses, String> {
    Ok(ModelStatuses {
        transcription: model_selection::transcription(app)?
            .transcription_models()
            .iter()
            .copied()
            .map(|kind| model_status(app, kind))
            .collect::<Result<Vec<_>, _>>()?,
        image_segmentation: model_status(
            app,
            model_selection::image_segmentation(app)?
                .snapshot()
                .expect("Image model owns a snapshot")
                .models()[0],
        )?,
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
        voxcpm2: models::catalog::VOXCPM2_MODELS
            .iter()
            .copied()
            .map(|kind| model_status(app, kind))
            .collect::<Result<Vec<_>, _>>()?,
        speech_alignment: models::catalog::SPEECH_ALIGNMENT_MODELS
            .iter()
            .copied()
            .map(|kind| model_status(app, kind))
            .collect::<Result<Vec<_>, _>>()?,
        speech_separation: models::catalog::SPEECH_SEPARATION_MODELS
            .iter()
            .copied()
            .map(|kind| model_status(app, kind))
            .collect::<Result<Vec<_>, _>>()?,
        diarization: model_selection::speech_diarization(app)?
            .snapshot()
            .expect("Diarization owns a snapshot")
            .models()
            .iter()
            .copied()
            .map(|kind| model_status(app, kind))
            .collect::<Result<Vec<_>, _>>()?,
    })
}

fn model_status(app: &AppHandle, model: ModelKind) -> Result<ModelStatus, String> {
    models::status(app, models::catalog::descriptor(model))
}

fn tool_status_from_path(path: Option<PathBuf>) -> ToolStatus {
    ToolStatus {
        available: path.is_some(),
        path: path.map(|value| value.to_string_lossy().into_owned()),
    }
}
