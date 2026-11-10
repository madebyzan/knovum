use crate::local_resources::models::catalog::ImageModelCatalog;
mod identity;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

pub use super::model_metadata::{
    LocalModelCapability, LocalModelOperation, LocalModelReferenceDuration,
    LocalModelReferenceTranscript,
};
use super::{
    model_metadata,
    models::{self, catalog, snapshots::SnapshotKind},
    preparation::contracts::ResourcePreparationTarget,
};

pub(crate) const VOXCPM2_FULL_SPEC_ID: &str = "voxcpm2-2b-gguf-orig-v1";
pub(crate) const VOXCPM2_BF16_SPEC_ID: &str = "voxcpm2-2b-gguf-bf16-v1";
pub(crate) const VOXCPM2_Q8_SPEC_ID: &str = "voxcpm2-2b-gguf-q8_0-v1";
pub const IMAGE_SEGMENTATION_BIREFNET_SPEC_ID: &str =
    "birefnet-general-lite-onnx:5600024376f572a557870a5eb0afb1e5961636bef4e1e22132025467d0f03333-v1";
pub const IMAGE_SEGMENTATION_RMBG20_SPEC_ID: &str =
    "rmbg-2.0-onnx:6db79b5ac18a5a7af3b32f25d8e2580d8dca444d8a77249d72700eed425f269f-v1";
pub const IMAGE_SEGMENTATION_PORTRAIT_SPEC_ID: &str =
    "birefnet-portrait-onnx:1ba1c8ff5a7bbfadc8d8d13fb11d7be793f91f23d9d466549e37a854f6668f99-v1";
pub const IMAGE_SEGMENTATION_RMBG14_SPEC_ID: &str =
    "rmbg-1.4-onnx:8cafcf770b06757c4eaced21b1a88e57fd2b66de01b8045f35f01535ba742e0f-v1";
pub const IMAGE_SEGMENTATION_DIS_SPEC_ID: &str =
    "isnet-general-use-onnx:60920e99c45464f2ba57bee2ad08c919a52bbf852739e96947fbb4358c0d964a-v1";
pub const IMAGE_SEGMENTATION_ANIME_SPEC_ID: &str =
    "isnet-anime-onnx:f15622d853e8260172812b657053460e20806f04b9e05147d49af7bed31a6e99-v1";

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, schemars::JsonSchema, ts_rs::TS,
)]
#[serde(rename_all = "camelCase")]
pub enum LocalModelId {
    Qwen3Embedding,
    WhisperSmall,
    WhisperTiny,
    WhisperBase,
    WhisperMedium,
    WhisperMediumFull,
    WhisperLargeV3TurboQ8,
    WhisperLargeV3Turbo,
    WhisperLargeV3,
    MossNano,
    VoxCpm2,
    VoxCpm2Bf16,
    VoxCpm2Q8,
    Qwen3Aligner,
    Qwen3AlignerQ8,
    BirefnetLite,
    Rmbg20,
    BirefnetPortrait,
    Rmbg14,
    IsnetDis,
    IsnetAnime,
    RealEsrganGeneralX4v3,
    RealEsrganX4plus,
    Migan,
    Lama,
    EfficientSamTi,
    EfficientSamS,
    DepthAnythingV2SmallUint8,
    DepthAnythingV2Small,
    PpOcrV6Small,
    Spleeter2Stems,
    Spleeter2StemsFull,
    SherpaDiarization,
    SherpaDiarizationCampplus,
    SherpaDiarizationFull,
    SherpaDiarizationEres2netLarge,
}

#[derive(Clone, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
pub struct LocalModelAvailability {
    pub id: LocalModelId,
    pub name: &'static str,
    pub capability: LocalModelCapability,
    pub operations: &'static [LocalModelOperation],
    pub reference_transcript: LocalModelReferenceTranscript,
    pub reference_duration: Option<LocalModelReferenceDuration>,
    pub group: LocalModelGroup,
    pub essential: bool,
    pub selected: bool,
    pub selectable: bool,
    pub fixed: bool,
    pub total_bytes: u64,
    pub installed_bytes: u64,
    pub installed: bool,
    pub verified: bool,
    // Set when this machine has less usable memory than the model was qualified against. It is a
    // caption, never a veto: the download is always allowed and a run is decided by admission.
    pub memory_advisory_bytes: Option<u64>,
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
    LocalModelId::PpOcrV6Small,
    LocalModelId::WhisperSmall,
    LocalModelId::WhisperTiny,
    LocalModelId::WhisperBase,
    LocalModelId::WhisperMedium,
    LocalModelId::WhisperMediumFull,
    LocalModelId::WhisperLargeV3TurboQ8,
    LocalModelId::WhisperLargeV3Turbo,
    LocalModelId::WhisperLargeV3,
    LocalModelId::MossNano,
    LocalModelId::VoxCpm2Q8,
    LocalModelId::VoxCpm2Bf16,
    LocalModelId::VoxCpm2,
    LocalModelId::Qwen3Aligner,
    LocalModelId::Qwen3AlignerQ8,
    LocalModelId::SherpaDiarization,
    LocalModelId::SherpaDiarizationCampplus,
    LocalModelId::SherpaDiarizationFull,
    LocalModelId::SherpaDiarizationEres2netLarge,
    LocalModelId::Spleeter2Stems,
    LocalModelId::Spleeter2StemsFull,
    LocalModelId::BirefnetLite,
    LocalModelId::Rmbg20,
    LocalModelId::BirefnetPortrait,
    LocalModelId::Rmbg14,
    LocalModelId::IsnetDis,
    LocalModelId::IsnetAnime,
    LocalModelId::RealEsrganGeneralX4v3,
    LocalModelId::RealEsrganX4plus,
    LocalModelId::Migan,
    LocalModelId::Lama,
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
        model_metadata::metadata(self).capability
    }
    pub(crate) fn name(self) -> &'static str {
        model_metadata::metadata(self).name
    }

    fn group(self) -> LocalModelGroup {
        match self {
            Self::Qwen3Embedding | Self::PpOcrV6Small => LocalModelGroup::Knowledge,
            Self::BirefnetLite
            | Self::Rmbg20
            | Self::BirefnetPortrait
            | Self::Rmbg14
            | Self::IsnetDis
            | Self::IsnetAnime
            | Self::RealEsrganGeneralX4v3
            | Self::RealEsrganX4plus
            | Self::Migan
            | Self::Lama
            | Self::EfficientSamTi
            | Self::EfficientSamS
            | Self::DepthAnythingV2SmallUint8
            | Self::DepthAnythingV2Small => LocalModelGroup::Images,
            Self::WhisperSmall
            | Self::WhisperTiny
            | Self::WhisperBase
            | Self::WhisperMedium
            | Self::WhisperMediumFull
            | Self::WhisperLargeV3TurboQ8
            | Self::WhisperLargeV3Turbo
            | Self::WhisperLargeV3
            | Self::MossNano
            | Self::VoxCpm2
            | Self::VoxCpm2Bf16
            | Self::VoxCpm2Q8
            | Self::Qwen3Aligner
            | Self::Qwen3AlignerQ8
            | Self::Spleeter2Stems
            | Self::Spleeter2StemsFull
            | Self::SherpaDiarization
            | Self::SherpaDiarizationCampplus
            | Self::SherpaDiarizationFull
            | Self::SherpaDiarizationEres2netLarge => LocalModelGroup::Audio,
        }
    }
    pub(crate) fn snapshot(self) -> Option<SnapshotKind> {
        use SnapshotKind as S;
        match self {
            Self::Qwen3Embedding => Some(S::TextEmbedding),
            Self::WhisperSmall
            | Self::WhisperTiny
            | Self::WhisperBase
            | Self::WhisperMedium
            | Self::WhisperMediumFull
            | Self::WhisperLargeV3TurboQ8
            | Self::WhisperLargeV3Turbo
            | Self::WhisperLargeV3 => Some(transcription_snapshot(self)),
            Self::MossNano => Some(S::MossNano),
            Self::VoxCpm2 => Some(S::VoxCpm2),
            Self::VoxCpm2Bf16 => Some(S::VoxCpm2Bf16),
            Self::VoxCpm2Q8 => Some(S::VoxCpm2Q8),
            Self::Qwen3Aligner => {
                (!catalog::SPEECH_ALIGNMENT_MODELS.is_empty()).then_some(S::SpeechAlignment)
            }
            Self::Qwen3AlignerQ8 => Some(S::SpeechAlignmentQ8),
            Self::BirefnetLite => Some(S::ImageSegmentationBiRefNet),
            Self::Rmbg20 => Some(S::ImageSegmentationRmbg20),
            Self::BirefnetPortrait => Some(S::ImageSegmentationPortrait),
            Self::Rmbg14 => Some(S::ImageSegmentationRmbg14),
            Self::IsnetDis => Some(S::ImageSegmentationDis),
            Self::IsnetAnime => Some(S::ImageSegmentationAnime),
            Self::Spleeter2Stems => Some(S::SpeechSeparation),
            Self::Spleeter2StemsFull => Some(S::SpeechSeparationFull),
            Self::SherpaDiarization => Some(S::Diarization(catalog::DiarizationVariant::Int8)),
            Self::SherpaDiarizationCampplus => {
                Some(S::Diarization(catalog::DiarizationVariant::Campplus))
            }
            Self::SherpaDiarizationFull => Some(S::Diarization(catalog::DiarizationVariant::Full)),
            Self::SherpaDiarizationEres2netLarge => {
                Some(S::Diarization(catalog::DiarizationVariant::Eres2netLarge))
            }
            Self::RealEsrganGeneralX4v3 => {
                Some(S::Image(catalog::ImageModelVariant::RealEsrganGeneralX4v3))
            }
            Self::RealEsrganX4plus => Some(S::Image(catalog::ImageModelVariant::RealEsrganX4plus)),
            Self::Migan => Some(S::Image(catalog::ImageModelVariant::Migan)),
            Self::Lama => Some(S::Image(catalog::ImageModelVariant::Lama)),
            Self::EfficientSamTi => Some(S::Image(catalog::ImageModelVariant::EfficientSamTi)),
            Self::EfficientSamS => Some(S::Image(catalog::ImageModelVariant::EfficientSamS)),
            Self::DepthAnythingV2SmallUint8 => Some(S::Image(
                catalog::ImageModelVariant::DepthAnythingV2SmallUint8,
            )),
            Self::DepthAnythingV2Small => {
                Some(S::Image(catalog::ImageModelVariant::DepthAnythingV2Small))
            }
            Self::PpOcrV6Small => Some(S::Image(catalog::ImageModelVariant::PpOcrV6Small)),
        }
    }

    pub(crate) fn preparation(self) -> ResourcePreparationTarget {
        match self {
            Self::Qwen3Embedding => ResourcePreparationTarget::TextEmbedding,
            Self::WhisperSmall => ResourcePreparationTarget::Transcription,
            Self::WhisperTiny
            | Self::WhisperBase
            | Self::WhisperMedium
            | Self::WhisperMediumFull
            | Self::WhisperLargeV3TurboQ8
            | Self::WhisperLargeV3Turbo
            | Self::WhisperLargeV3
            | Self::RealEsrganGeneralX4v3
            | Self::RealEsrganX4plus
            | Self::Migan
            | Self::Lama
            | Self::EfficientSamTi
            | Self::EfficientSamS
            | Self::DepthAnythingV2SmallUint8
            | Self::DepthAnythingV2Small
            | Self::PpOcrV6Small
            | Self::Rmbg20
            | Self::BirefnetPortrait
            | Self::Rmbg14
            | Self::IsnetDis
            | Self::IsnetAnime
            | Self::VoxCpm2Bf16
            | Self::VoxCpm2Q8
            | Self::Qwen3AlignerQ8
            | Self::Spleeter2StemsFull
            | Self::SherpaDiarizationCampplus
            | Self::SherpaDiarizationFull
            | Self::SherpaDiarizationEres2netLarge => {
                ResourcePreparationTarget::Model { model: self }
            }
            Self::MossNano => ResourcePreparationTarget::SpeechSynthesis,
            Self::VoxCpm2 => ResourcePreparationTarget::VoiceDesign,
            Self::Qwen3Aligner => ResourcePreparationTarget::SpeechAlignment,
            Self::BirefnetLite => ResourcePreparationTarget::ImageSegmentation,
            Self::Spleeter2Stems => ResourcePreparationTarget::SpeechSeparation,
            Self::SherpaDiarization => ResourcePreparationTarget::SpeechDiarization,
        }
    }
}

pub(crate) fn list<R: Runtime>(app: &AppHandle<R>) -> Result<Vec<LocalModelAvailability>, String> {
    MODELS.iter().map(|&id| inspect(app, id)).collect()
}

pub(crate) fn inspect<R: Runtime>(
    app: &AppHandle<R>,
    model_id: LocalModelId,
) -> Result<LocalModelAvailability, String> {
    let metadata = model_metadata::metadata(model_id);
    let snapshot = model_id
        .snapshot()
        .ok_or("Local model is unavailable on this platform")?;
    let parts = snapshot
        .models()
        .iter()
        .map(|kind| models::status(app, catalog::descriptor(*kind)))
        .collect::<Result<Vec<_>, _>>()?;
    // A machine we could not measure produces no advisory, never a claim that it is too small.
    let usable_memory = super::system_memory::physical_memory().ok();
    let transcription = super::model_selection::transcription(app)?;
    Ok(LocalModelAvailability {
        id: model_id,
        name: metadata.name,
        capability: metadata.capability,
        operations: metadata.operations,
        reference_transcript: metadata.reference_transcript,
        reference_duration: metadata.reference_duration,
        group: model_id.group(),
        selected: super::model_selection::is_selected(app, model_id)?,
        selectable: super::model_selection::is_selectable(model_id),
        essential: model_id.is_essential(transcription),
        fixed: matches!(
            model_id,
            LocalModelId::Qwen3Embedding | LocalModelId::PpOcrV6Small
        ),
        total_bytes: snapshot.total_bytes(),
        installed_bytes: parts.iter().filter_map(|part| part.size).sum(),
        installed: parts.iter().any(|part| part.installed),
        verified: parts.iter().all(|part| part.verified),
        memory_advisory_bytes: metadata.memory.advisory_bytes(usable_memory),
    })
}

#[cfg(test)]
mod tests;

fn transcription_snapshot(model: LocalModelId) -> SnapshotKind {
    let variant = match model {
        LocalModelId::WhisperTiny => catalog::WhisperVariant::Tiny,
        LocalModelId::WhisperBase => catalog::WhisperVariant::Base,
        LocalModelId::WhisperSmall => catalog::WhisperVariant::Small,
        LocalModelId::WhisperMedium => catalog::WhisperVariant::Medium,
        LocalModelId::WhisperMediumFull => catalog::WhisperVariant::MediumFull,
        LocalModelId::WhisperLargeV3TurboQ8 => catalog::WhisperVariant::LargeV3TurboQ8,
        LocalModelId::WhisperLargeV3Turbo => catalog::WhisperVariant::LargeV3Turbo,
        LocalModelId::WhisperLargeV3 => catalog::WhisperVariant::LargeV3,
        _ => unreachable!("The catalog owns supported transcription models"),
    };
    SnapshotKind::Whisper(variant)
}
