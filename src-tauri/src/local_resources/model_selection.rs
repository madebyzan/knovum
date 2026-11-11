use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::StoreExt;

use super::model_catalog::{self, LocalModelCapability, LocalModelId};
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

            Self::SpeechSeparation => LocalModelId::HtDemucs,
            Self::SpeechDiarization => LocalModelId::SherpaDiarization,
            Self::SpeechAlignment => LocalModelId::Qwen3AlignerQ8,
        }
    }
}

fn group(model: LocalModelId) -> Option<Group> {
    match model {
        LocalModelId::WhisperSmall
        | LocalModelId::WhisperTiny
        | LocalModelId::WhisperBase
        | LocalModelId::WhisperMedium
        | LocalModelId::WhisperMediumFull
        | LocalModelId::WhisperLargeV3TurboQ8
        | LocalModelId::WhisperLargeV3Turbo
        | LocalModelId::WhisperLargeV3 => Some(Group::Transcription),
        LocalModelId::MossNano
        | LocalModelId::VoxCpm2
        | LocalModelId::VoxCpm2Bf16
        | LocalModelId::VoxCpm2Q8
        | LocalModelId::IndexTts2
        | LocalModelId::IndexTts2F16
        | LocalModelId::Qwen3Tts
        | LocalModelId::Qwen3TtsBf16
        | LocalModelId::Qwen3TtsSmall
        | LocalModelId::CosyVoice3 => Some(Group::SpeechSynthesis),
        LocalModelId::RealEsrganGeneralX4v3 | LocalModelId::RealEsrganX4plus => {
            Some(Group::ImageUpscaling)
        }
        LocalModelId::Migan | LocalModelId::Lama => Some(Group::ImageInpainting),
        LocalModelId::EfficientSamTi | LocalModelId::EfficientSamS => Some(Group::ImageSelection),
        LocalModelId::DepthAnythingV2SmallUint8 | LocalModelId::DepthAnythingV2Small => {
            Some(Group::ImageDepth)
        }
        LocalModelId::U2netp
        | LocalModelId::Silueta
        | LocalModelId::U2netHumanSeg
        | LocalModelId::U2net => Some(Group::ImageSegmentation),
        LocalModelId::HtDemucs | LocalModelId::HtDemucsF16 => Some(Group::SpeechSeparation),
        LocalModelId::SherpaDiarization
        | LocalModelId::SherpaDiarizationCampplus
        | LocalModelId::SherpaDiarizationFull
        | LocalModelId::SherpaDiarizationEres2netLarge => Some(Group::SpeechDiarization),
        LocalModelId::Qwen3Aligner | LocalModelId::Qwen3AlignerQ8 => Some(Group::SpeechAlignment),
        _ => None,
    }
}

/// Models a person could once choose and the catalogue has since retired, by their persisted id.
/// A stored choice of one is no choice any more: the capability uses its default, and a backup
/// carrying one restores. Every other unknown id is a corrupt preference.
const RETIRED: &[(&str, Group)] = &[
    ("spleeter2Stems", Group::SpeechSeparation),
    ("spleeter2StemsFull", Group::SpeechSeparation),
];

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
    chosen(&value, expected).map(drop)
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Selection<Value> {
    schema_version: u8,
    value: Value,
}

/// The model a persisted model id names for `expected`, or `None` when it names a retired one.
fn chosen(value: &serde_json::Value, expected: Group) -> Result<Option<LocalModelId>, String> {
    if RETIRED
        .iter()
        .any(|(id, group)| *group == expected && value.as_str() == Some(*id))
    {
        return Ok(None);
    }
    let model: LocalModelId = serde_json::from_value(value.clone())
        .map_err(|error| format!("Invalid local model selection: {error}"))?;
    if group(model) != Some(expected) {
        return Err("Invalid local model selection".into());
    }
    Ok(Some(model))
}

fn read_selection(raw: serde_json::Value, expected: Group) -> Result<LocalModelId, String> {
    let selection: Selection<serde_json::Value> = serde_json::from_value(raw)
        .map_err(|error| format!("Invalid local model selection: {error}"))?;
    if selection.schema_version != 1 {
        return Err("Invalid local model selection".into());
    }
    Ok(chosen(&selection.value, expected)?.unwrap_or(expected.default_model()))
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

/// The model a person chose for one image capability. Every image derivation reads its models from
/// here, so a choice made in Settings is the model that runs wherever that capability is used.
pub(crate) fn image<R: Runtime>(
    app: &AppHandle<R>,
    capability: LocalModelCapability,
) -> Result<LocalModelId, String> {
    selected(
        app,
        image_group(capability).ok_or("This capability has no image model choice")?,
    )
}

fn image_group(capability: LocalModelCapability) -> Option<Group> {
    use LocalModelCapability as C;
    match capability {
        C::ImageSegmentation => Some(Group::ImageSegmentation),
        C::ImageUpscaling => Some(Group::ImageUpscaling),
        C::ImageInpainting => Some(Group::ImageInpainting),
        C::ImageSelection => Some(Group::ImageSelection),
        C::ImageDepth => Some(Group::ImageDepth),
        _ => None,
    }
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
        let status = model_catalog::inspect(&app, model)?;
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
