use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

use super::model_catalog::{self, LocalModelId};
use crate::foundation::app_error::AppError;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Group {
    Transcription,
    SpeechSynthesis,
    ImageSegmentation,
    ImageUpscaling,
    ImageInpainting,
    ImageSelection,
    ImageDepth,
    SpeechSeparation,
    SpeechDiarization,
    SpeechAlignment,
}

impl Group {
    fn key(self) -> &'static str {
        match self {
            Self::Transcription => "localResources.transcription",
            Self::SpeechSynthesis => "localResources.speechSynthesis",
            Self::ImageSegmentation => "localResources.imageSegmentation",
            Self::ImageUpscaling => "localResources.imageUpscaling",
            Self::ImageInpainting => "localResources.imageInpainting",
            Self::ImageSelection => "localResources.imageSelection",
            Self::ImageDepth => "localResources.imageDepth",

            Self::SpeechSeparation => "localResources.speechSeparation",
            Self::SpeechDiarization => "localResources.speechDiarization",
            Self::SpeechAlignment => "localResources.speechAlignment",
        }
    }
    fn default_model(self) -> LocalModelId {
        match self {
            Self::Transcription => LocalModelId::WhisperSmall,
            Self::SpeechSynthesis => LocalModelId::MossNano,
            Self::ImageSegmentation => LocalModelId::U2netp,
            Self::ImageUpscaling => LocalModelId::RealEsrganGeneralX4v3,
            Self::ImageInpainting => LocalModelId::Migan,
            Self::ImageSelection => LocalModelId::EfficientSamTi,
            Self::ImageDepth => LocalModelId::DepthAnythingV2Small,

            Self::SpeechSeparation => LocalModelId::Spleeter2Stems,
            Self::SpeechDiarization => LocalModelId::SherpaDiarization,
            Self::SpeechAlignment => LocalModelId::Qwen3AlignerQ8,
        }
    }
}

fn group(model: LocalModelId) -> Option<Group> {
    match model {
        LocalModelId::WhisperSmall | LocalModelId::WhisperTiny | LocalModelId::WhisperLargeV3 => {
            Some(Group::Transcription)
        }
        LocalModelId::MossNano | LocalModelId::VoxCpm2 | LocalModelId::VoxCpm2Q8 => {
            Some(Group::SpeechSynthesis)
        }
        LocalModelId::RealEsrganGeneralX4v3 | LocalModelId::RealEsrganX4plus => {
            Some(Group::ImageUpscaling)
        }
        LocalModelId::Migan => Some(Group::ImageInpainting),
        LocalModelId::EfficientSamTi | LocalModelId::EfficientSamS => Some(Group::ImageSelection),
        LocalModelId::DepthAnythingV2SmallUint8 | LocalModelId::DepthAnythingV2Small => {
            Some(Group::ImageDepth)
        }
        LocalModelId::U2netp | LocalModelId::U2net => Some(Group::ImageSegmentation),
        LocalModelId::Spleeter2Stems | LocalModelId::Spleeter2StemsFull => {
            Some(Group::SpeechSeparation)
        }
        LocalModelId::SherpaDiarization | LocalModelId::SherpaDiarizationFull => {
            Some(Group::SpeechDiarization)
        }
        LocalModelId::Qwen3Aligner | LocalModelId::Qwen3AlignerQ8 => Some(Group::SpeechAlignment),
        _ => None,
    }
}

pub(crate) fn validate_persisted_value(key: &str, value: serde_json::Value) -> Result<(), String> {
    let expected = [
        Group::Transcription,
        Group::SpeechSynthesis,
        Group::ImageSegmentation,
        Group::ImageUpscaling,
        Group::ImageInpainting,
        Group::ImageSelection,
        Group::ImageDepth,
        Group::SpeechSeparation,
        Group::SpeechDiarization,
        Group::SpeechAlignment,
    ]
    .into_iter()
    .find(|group| group.key() == key)
    .ok_or("Unknown local model selection")?;
    let model: LocalModelId = serde_json::from_value(value)
        .map_err(|error| format!("Invalid local model selection: {error}"))?;
    if group(model) != Some(expected) {
        return Err("Invalid local model selection".into());
    }
    Ok(())
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Selection {
    schema_version: u8,
    value: LocalModelId,
}

fn read_selection(raw: serde_json::Value, expected: Group) -> Result<LocalModelId, String> {
    let selection: Selection = serde_json::from_value(raw)
        .map_err(|error| format!("Invalid local model selection: {error}"))?;
    if selection.schema_version != 1 || group(selection.value) != Some(expected) {
        return Err("Invalid local model selection".into());
    }
    Ok(selection.value)
}

fn selected<R: Runtime>(app: &AppHandle<R>, group: Group) -> Result<LocalModelId, String> {
    let store = app
        .store("settings.json")
        .map_err(|error| error.to_string())?;
    match store.get(group.key()) {
        Some(raw) => read_selection(raw, group),
        None => Ok(group.default_model()),
    }
}

pub(crate) fn transcription<R: Runtime>(app: &AppHandle<R>) -> Result<LocalModelId, String> {
    selected(app, Group::Transcription)
}

pub(crate) fn image_segmentation<R: Runtime>(app: &AppHandle<R>) -> Result<LocalModelId, String> {
    selected(app, Group::ImageSegmentation)
}

pub(crate) fn speech_separation<R: Runtime>(app: &AppHandle<R>) -> Result<LocalModelId, String> {
    selected(app, Group::SpeechSeparation)
}

pub(crate) fn speech_diarization<R: Runtime>(app: &AppHandle<R>) -> Result<LocalModelId, String> {
    selected(app, Group::SpeechDiarization)
}

pub(crate) fn speech_alignment<R: Runtime>(app: &AppHandle<R>) -> Result<LocalModelId, String> {
    selected(app, Group::SpeechAlignment)
}

pub(crate) fn is_selectable(model: LocalModelId) -> bool {
    group(model).is_some()
}

pub(crate) fn is_selected<R: Runtime>(
    app: &AppHandle<R>,
    model: LocalModelId,
) -> Result<bool, String> {
    match group(model) {
        Some(group) => Ok(selected(app, group)? == model),
        None => Ok(false),
    }
}

#[tauri::command]
pub async fn select_local_model(app: AppHandle, model: LocalModelId) -> Result<(), AppError> {
    let group =
        group(model).ok_or_else(|| AppError::invalid("This model has a fixed capability"))?;
    let _usage = super::preparation::locks::resource_lock(
        model
            .snapshot()
            .expect("Selectable models own a complete snapshot"),
    )?
    .read_owned()
    .await;
    crate::foundation::blocking::run(move || {
        let status = model_catalog::inspect(&app, model, &|_| Ok(None))?;
        if !status.verified {
            return Err("Download and verify the model before selecting it".into());
        }
        let store = app
            .store("settings.json")
            .map_err(|error| error.to_string())?;
        let selection = serde_json::to_value(Selection {
            schema_version: 1,
            value: model,
        })
        .map_err(|error| error.to_string())?;
        store.set(group.key(), selection);
        store.save().map_err(|error| error.to_string())
    })
    .await?;
    Ok(())
}

#[cfg(test)]
#[path = "tests/model_selection.rs"]
mod tests;
