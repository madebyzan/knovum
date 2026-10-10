mod identity;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

pub use super::model_metadata::{
    LocalModelCapability, LocalModelOperation, LocalModelReferenceTranscript,
};
use super::{
    model_metadata,
    models::{self, catalog, snapshots::SnapshotKind},
    preparation::contracts::ResourcePreparationTarget,
};

pub(crate) const VOXCPM2_FULL_SPEC_ID: &str = "voxcpm2-2b-gguf-orig-v1";
pub(crate) const VOXCPM2_Q8_SPEC_ID: &str = "voxcpm2-2b-gguf-q8_0-v1";
pub const IMAGE_SEGMENTATION_SPEC_ID: &str =
    "u2netp-onnx:309c8469258dda742793dce0ebea8e6dd393174f89934733ecc8b14c76f4ddd8-v1";
pub const IMAGE_SEGMENTATION_QUALITY_SPEC_ID: &str =
    "u2net-onnx:8d10d2f3bb75ae3b6d527c77944fc5e7dcd94b29809d47a739a7a728a912b491-v1";

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema, ts_rs::TS,
)]
#[serde(rename_all = "camelCase")]
pub enum LocalModelId {
    Qwen3Embedding,
    WhisperSmall,
    WhisperTiny,
    WhisperLargeV3,
    MossNano,
    VoxCpm2,
    VoxCpm2Q8,
    Qwen3Aligner,
    Qwen3AlignerQ8,
    U2netp,
    U2net,
    RealEsrganGeneralX4v3,
    RealEsrganX4plus,
    Migan,
    EfficientSamTi,
    EfficientSamS,
    DepthAnythingV2SmallUint8,
    DepthAnythingV2Small,
    Spleeter2Stems,
    Spleeter2StemsFull,
    SherpaDiarization,
    SherpaDiarizationFull,
}

#[derive(Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct LocalModelAvailability {
    pub id: LocalModelId,
    pub name: &'static str,
    pub capability: LocalModelCapability,
    pub operations: &'static [LocalModelOperation],
    pub reference_transcript: LocalModelReferenceTranscript,
    pub group: LocalModelGroup,
    pub essential: bool,
    pub selected: bool,
    pub selectable: bool,
    pub fixed: bool,
    pub total_bytes: u64,
    pub installed_bytes: u64,
    pub installed: bool,
    pub verified: bool,
    pub download_available: bool,
    pub unavailable_reason: Option<&'static str>,
}

#[derive(Clone, Copy, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub enum LocalModelGroup {
    Knowledge,
    Audio,
    Images,
}

pub const MODELS: &[LocalModelId] = &[
    LocalModelId::Qwen3Embedding,
    LocalModelId::WhisperSmall,
    LocalModelId::WhisperTiny,
    LocalModelId::WhisperLargeV3,
    LocalModelId::MossNano,
    LocalModelId::VoxCpm2Q8,
    LocalModelId::VoxCpm2,
    LocalModelId::Qwen3Aligner,
    LocalModelId::Qwen3AlignerQ8,
    LocalModelId::SherpaDiarization,
    LocalModelId::SherpaDiarizationFull,
    LocalModelId::Spleeter2Stems,
    LocalModelId::Spleeter2StemsFull,
    LocalModelId::U2netp,
    LocalModelId::U2net,
    LocalModelId::RealEsrganGeneralX4v3,
    LocalModelId::RealEsrganX4plus,
    LocalModelId::Migan,
    LocalModelId::EfficientSamTi,
    LocalModelId::EfficientSamS,
    LocalModelId::DepthAnythingV2SmallUint8,
    LocalModelId::DepthAnythingV2Small,
];

impl LocalModelId {
    pub(crate) fn image_spec(self) -> Result<String, String> {
        match self.snapshot() {
            Some(SnapshotKind::Image(variant)) => Ok(variant.spec_id()),
            _ => self.image_segmentation_spec().map(str::to_owned),
        }
    }
    pub(crate) fn capability(self) -> LocalModelCapability {
        model_metadata::metadata(self).1
    }
    pub(crate) fn name(self) -> &'static str {
        model_metadata::metadata(self).0
    }

    fn group(self) -> LocalModelGroup {
        match self {
            Self::Qwen3Embedding => LocalModelGroup::Knowledge,
            Self::U2netp
            | Self::U2net
            | Self::RealEsrganGeneralX4v3
            | Self::RealEsrganX4plus
            | Self::Migan
            | Self::EfficientSamTi
            | Self::EfficientSamS
            | Self::DepthAnythingV2SmallUint8
            | Self::DepthAnythingV2Small => LocalModelGroup::Images,
            Self::WhisperSmall
            | Self::WhisperTiny
            | Self::WhisperLargeV3
            | Self::MossNano
            | Self::VoxCpm2
            | Self::VoxCpm2Q8
            | Self::Qwen3Aligner
            | Self::Qwen3AlignerQ8
            | Self::Spleeter2Stems
            | Self::Spleeter2StemsFull
            | Self::SherpaDiarization
            | Self::SherpaDiarizationFull => LocalModelGroup::Audio,
        }
    }
    pub(crate) fn snapshot(self) -> Option<SnapshotKind> {
        use SnapshotKind as S;
        match self {
            Self::Qwen3Embedding => Some(S::TextEmbedding),
            Self::WhisperSmall => catalog::TRANSCRIPTION_MODELS
                .first()
                .map(|kind| catalog::descriptor(*kind).snapshot),
            Self::WhisperTiny | Self::WhisperLargeV3 => Some(transcription_snapshot(self)),
            Self::MossNano => Some(S::MossNano),
            Self::VoxCpm2 => Some(S::VoxCpm2),
            Self::VoxCpm2Q8 => Some(S::VoxCpm2Q8),
            Self::Qwen3Aligner => {
                (!catalog::SPEECH_ALIGNMENT_MODELS.is_empty()).then_some(S::SpeechAlignment)
            }
            Self::Qwen3AlignerQ8 => Some(S::SpeechAlignmentQ8),
            Self::U2netp => Some(S::ImageSegmentation),
            Self::U2net => Some(S::ImageSegmentationQuality),
            Self::Spleeter2Stems => Some(S::SpeechSeparation),
            Self::Spleeter2StemsFull => Some(S::SpeechSeparationFull),
            Self::SherpaDiarization => Some(S::Diarization(catalog::DiarizationVariant::Int8)),
            Self::SherpaDiarizationFull => Some(S::Diarization(catalog::DiarizationVariant::Full)),
            Self::RealEsrganGeneralX4v3 => {
                Some(S::Image(catalog::ImageModelVariant::RealEsrganGeneralX4v3))
            }
            Self::RealEsrganX4plus => Some(S::Image(catalog::ImageModelVariant::RealEsrganX4plus)),
            Self::Migan => Some(S::Image(catalog::ImageModelVariant::Migan)),
            Self::EfficientSamTi => Some(S::Image(catalog::ImageModelVariant::EfficientSamTi)),
            Self::EfficientSamS => Some(S::Image(catalog::ImageModelVariant::EfficientSamS)),
            Self::DepthAnythingV2SmallUint8 => Some(S::Image(
                catalog::ImageModelVariant::DepthAnythingV2SmallUint8,
            )),
            Self::DepthAnythingV2Small => {
                Some(S::Image(catalog::ImageModelVariant::DepthAnythingV2Small))
            }
        }
    }

    pub(crate) fn preparation(self) -> ResourcePreparationTarget {
        match self {
            Self::Qwen3Embedding => ResourcePreparationTarget::TextEmbedding,
            Self::WhisperSmall => ResourcePreparationTarget::Transcription,
            Self::WhisperTiny
            | Self::WhisperLargeV3
            | Self::RealEsrganGeneralX4v3
            | Self::RealEsrganX4plus
            | Self::Migan
            | Self::EfficientSamTi
            | Self::EfficientSamS
            | Self::DepthAnythingV2SmallUint8
            | Self::DepthAnythingV2Small
            | Self::U2net
            | Self::VoxCpm2Q8
            | Self::Qwen3AlignerQ8
            | Self::Spleeter2StemsFull
            | Self::SherpaDiarizationFull => ResourcePreparationTarget::Model { model: self },
            Self::MossNano => ResourcePreparationTarget::SpeechSynthesis,
            Self::VoxCpm2 => ResourcePreparationTarget::VoiceDesign,
            Self::Qwen3Aligner => ResourcePreparationTarget::SpeechAlignment,
            Self::U2netp => ResourcePreparationTarget::ImageSegmentation,
            Self::Spleeter2Stems => ResourcePreparationTarget::SpeechSeparation,
            Self::SherpaDiarization => ResourcePreparationTarget::SpeechDiarization,
        }
    }
}

pub(crate) fn list<R: Runtime, F>(
    app: &AppHandle<R>,
    voice_eligible: F,
) -> Result<Vec<LocalModelAvailability>, String>
where
    F: Fn(LocalModelId) -> Result<bool, String>,
{
    MODELS
        .iter()
        .map(|&id| inspect(app, id, &voice_eligible))
        .collect()
}

pub(crate) fn inspect<R: Runtime, F>(
    app: &AppHandle<R>,
    model_id: LocalModelId,
    voice_eligible: &F,
) -> Result<LocalModelAvailability, String>
where
    F: Fn(LocalModelId) -> Result<bool, String>,
{
    let (name, capability, operations, reference_transcript) = model_metadata::metadata(model_id);
    let snapshot = model_id
        .snapshot()
        .ok_or("Local model is unavailable on this platform")?;
    let parts = snapshot
        .models()
        .iter()
        .map(|kind| models::status(app, catalog::descriptor(*kind)))
        .collect::<Result<Vec<_>, _>>()?;
    let unavailable_reason = availability_reason(model_id, voice_eligible)?;
    let transcription = super::model_selection::transcription(app)?;
    Ok(LocalModelAvailability {
        id: model_id,
        name,
        capability,
        operations,
        reference_transcript,
        group: model_id.group(),
        selected: super::model_selection::is_selected(app, model_id)?,
        selectable: super::model_selection::is_selectable(model_id),
        essential: model_id.is_essential(transcription),
        fixed: model_id == LocalModelId::Qwen3Embedding,
        total_bytes: snapshot.total_bytes(),
        installed_bytes: parts.iter().filter_map(|part| part.size).sum(),
        installed: parts.iter().any(|part| part.installed),
        verified: parts.iter().all(|part| part.verified),
        download_available: unavailable_reason.is_none(),
        unavailable_reason,
    })
}

fn availability_reason<F>(
    model_id: LocalModelId,
    voice_eligible: F,
) -> Result<Option<&'static str>, String>
where
    F: Fn(LocalModelId) -> Result<bool, String>,
{
    if matches!(model_id, LocalModelId::VoxCpm2 | LocalModelId::VoxCpm2Q8)
        && !voice_eligible(model_id)?
    {
        return Ok(Some("memory"));
    }
    Ok(None)
}

#[cfg(test)]
#[path = "../tests/model_catalog.rs"]
mod tests;

fn transcription_snapshot(model: LocalModelId) -> SnapshotKind {
    use catalog::WhisperVariant as V;
    #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
    let variant = match model {
        LocalModelId::WhisperTiny => V::MlxTiny,
        LocalModelId::WhisperLargeV3 => V::MlxLargeV3,
        _ => unreachable!("The catalog owns supported transcription alternatives"),
    };
    #[cfg(not(all(target_os = "macos", target_arch = "aarch64")))]
    let variant = match model {
        LocalModelId::WhisperTiny => V::FasterTiny,
        LocalModelId::WhisperLargeV3 => V::FasterLargeV3,
        _ => unreachable!("The catalog owns supported transcription alternatives"),
    };
    SnapshotKind::Whisper(variant)
}
