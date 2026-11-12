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
    Qwen3Asr,
    Qwen3AsrF16,
    MossNano,
    VoxCpm2,
    VoxCpm2Bf16,
    VoxCpm2Q8,
    IndexTts2,
    IndexTts2F16,
    Qwen3Tts,
    Qwen3TtsBf16,
    Qwen3TtsSmall,
    CosyVoice3,
    Qwen3Aligner,
    Qwen3AlignerQ8,
    BirefnetLite,
    BirefnetGeneral,
    BirefnetPortrait,
    Ben2Base,
    IsnetAnime,
    RealEsrganGeneralX4v3,
    RealEsrganX4plus,
    Lama,
    MobileSam,
    Sam2HieraTiny,
    DepthAnythingV2SmallUint8,
    DepthAnythingV2Small,
    PpOcrV6Small,
    HtDemucs,
    HtDemucsF16,
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
    LocalModelId::Qwen3Asr,
    LocalModelId::Qwen3AsrF16,
    LocalModelId::MossNano,
    LocalModelId::VoxCpm2Q8,
    LocalModelId::VoxCpm2Bf16,
    LocalModelId::VoxCpm2,
    LocalModelId::IndexTts2,
    LocalModelId::IndexTts2F16,
    LocalModelId::Qwen3Tts,
    LocalModelId::Qwen3TtsBf16,
    LocalModelId::Qwen3TtsSmall,
    LocalModelId::CosyVoice3,
    LocalModelId::Qwen3Aligner,
    LocalModelId::Qwen3AlignerQ8,
    LocalModelId::SherpaDiarization,
    LocalModelId::SherpaDiarizationCampplus,
    LocalModelId::SherpaDiarizationFull,
    LocalModelId::SherpaDiarizationEres2netLarge,
    LocalModelId::HtDemucs,
    LocalModelId::HtDemucsF16,
    LocalModelId::BirefnetLite,
    LocalModelId::BirefnetGeneral,
    LocalModelId::BirefnetPortrait,
    LocalModelId::Ben2Base,
    LocalModelId::IsnetAnime,
    LocalModelId::RealEsrganGeneralX4v3,
    LocalModelId::RealEsrganX4plus,
    LocalModelId::Lama,
    LocalModelId::MobileSam,
    LocalModelId::Sam2HieraTiny,
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
            | Self::BirefnetGeneral
            | Self::BirefnetPortrait
            | Self::Ben2Base
            | Self::IsnetAnime
            | Self::RealEsrganGeneralX4v3
            | Self::RealEsrganX4plus
            | Self::Lama
            | Self::MobileSam
            | Self::Sam2HieraTiny
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
            | Self::Qwen3Asr
            | Self::Qwen3AsrF16
            | Self::MossNano
            | Self::VoxCpm2
            | Self::VoxCpm2Bf16
            | Self::VoxCpm2Q8
            | Self::IndexTts2
            | Self::IndexTts2F16
            | Self::Qwen3Tts
            | Self::Qwen3TtsBf16
            | Self::Qwen3TtsSmall
            | Self::CosyVoice3
            | Self::Qwen3Aligner
            | Self::Qwen3AlignerQ8
            | Self::HtDemucs
            | Self::HtDemucsF16
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
            Self::Qwen3Asr | Self::Qwen3AsrF16 => self.qwen3_asr_variant().map(S::Qwen3Asr),
            Self::MossNano => Some(S::MossNano),
            Self::VoxCpm2 => Some(S::VoxCpm2),
            Self::VoxCpm2Bf16 => Some(S::VoxCpm2Bf16),
            Self::VoxCpm2Q8 => Some(S::VoxCpm2Q8),
            Self::IndexTts2
            | Self::IndexTts2F16
            | Self::Qwen3Tts
            | Self::Qwen3TtsBf16
            | Self::Qwen3TtsSmall
            | Self::CosyVoice3 => self.voice_variant().map(S::Voice),
            Self::Qwen3Aligner => {
                (!catalog::SPEECH_ALIGNMENT_MODELS.is_empty()).then_some(S::SpeechAlignment)
            }
            Self::Qwen3AlignerQ8 => Some(S::SpeechAlignmentQ8),
            Self::BirefnetLite
            | Self::BirefnetGeneral
            | Self::BirefnetPortrait
            | Self::Ben2Base
            | Self::IsnetAnime => self.segmentation().map(S::ImageSegmentation),
            Self::HtDemucs | Self::HtDemucsF16 => self.separation().map(S::Separation),
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
            Self::Lama => Some(S::Image(catalog::ImageModelVariant::Lama)),
            Self::MobileSam => Some(S::Image(catalog::ImageModelVariant::MobileSam)),
            Self::Sam2HieraTiny => Some(S::Image(catalog::ImageModelVariant::Sam2HieraTiny)),
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
            | Self::Qwen3Asr
            | Self::Qwen3AsrF16
            | Self::RealEsrganGeneralX4v3
            | Self::RealEsrganX4plus
            | Self::Lama
            | Self::MobileSam
            | Self::Sam2HieraTiny
            | Self::DepthAnythingV2SmallUint8
            | Self::DepthAnythingV2Small
            | Self::PpOcrV6Small
            | Self::BirefnetGeneral
            | Self::BirefnetPortrait
            | Self::Ben2Base
            | Self::IsnetAnime
            | Self::VoxCpm2Bf16
            | Self::VoxCpm2Q8
            | Self::IndexTts2
            | Self::IndexTts2F16
            | Self::Qwen3Tts
            | Self::Qwen3TtsBf16
            | Self::Qwen3TtsSmall
            | Self::CosyVoice3
            | Self::Qwen3AlignerQ8
            | Self::HtDemucsF16
            | Self::SherpaDiarizationCampplus
            | Self::SherpaDiarizationFull
            | Self::SherpaDiarizationEres2netLarge => {
                ResourcePreparationTarget::Model { model: self }
            }
            Self::MossNano => ResourcePreparationTarget::SpeechSynthesis,
            Self::VoxCpm2 => ResourcePreparationTarget::VoiceDesign,
            Self::Qwen3Aligner => ResourcePreparationTarget::SpeechAlignment,
            Self::BirefnetLite => ResourcePreparationTarget::ImageSegmentation,
            Self::HtDemucs => ResourcePreparationTarget::SpeechSeparation,
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
