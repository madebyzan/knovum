use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

pub use super::model_metadata::{LocalModelOperation, LocalModelReferenceTranscript};
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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, ts_rs::TS)]
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
    pub capability: &'static str,
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
];

impl LocalModelId {
    fn group(self) -> LocalModelGroup {
        match self {
            Self::Qwen3Embedding => LocalModelGroup::Knowledge,
            Self::U2netp | Self::U2net => LocalModelGroup::Images,
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
        if self == Self::Qwen3AlignerQ8 {
            return Some(SnapshotKind::SpeechAlignmentQ8);
        }
        if self == Self::VoxCpm2Q8 {
            return Some(SnapshotKind::VoxCpm2Q8);
        }
        if self == Self::U2net {
            return Some(SnapshotKind::ImageSegmentationQuality);
        }
        if matches!(self, Self::WhisperTiny | Self::WhisperLargeV3) {
            return Some(transcription_snapshot(self));
        }
        self.preparation()
            .models()
            .and_then(|kinds| kinds.first())
            .map(|kind| catalog::descriptor(*kind).snapshot)
    }

    pub(crate) fn preparation(self) -> ResourcePreparationTarget {
        match self {
            Self::Qwen3Embedding => ResourcePreparationTarget::TextEmbedding,
            Self::WhisperSmall => ResourcePreparationTarget::Transcription,
            Self::WhisperTiny
            | Self::WhisperLargeV3
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
#[path = "tests/model_catalog.rs"]
mod tests;

impl LocalModelId {
    pub(crate) fn alignment_spec(self) -> Result<&'static str, String> {
        match self {
            Self::Qwen3Aligner => Ok("qwen3-forced-aligner-0.6b-hf-v1"),
            Self::Qwen3AlignerQ8 => Ok("qwen3-forced-aligner-0.6b-q8_0-v1"),
            _ => Err("Model does not support forced alignment".into()),
        }
    }
    pub(crate) fn voxcpm2_spec(self) -> Result<&'static str, String> {
        match self {
            Self::VoxCpm2 => Ok(VOXCPM2_FULL_SPEC_ID),
            Self::VoxCpm2Q8 => Ok(VOXCPM2_Q8_SPEC_ID),
            _ => Err("Model does not support VoxCPM2 synthesis".into()),
        }
    }
    pub(crate) fn from_voxcpm2_spec(spec: &str) -> Result<Self, String> {
        [Self::VoxCpm2, Self::VoxCpm2Q8]
            .into_iter()
            .find(|model| model.voxcpm2_spec().is_ok_and(|value| value == spec))
            .ok_or("runtime_mismatch: VoxCPM2 model specification is unsupported".into())
    }
    pub(crate) fn voxcpm2_models(self) -> Result<&'static [catalog::ModelKind], String> {
        match self {
            Self::VoxCpm2 => Ok(catalog::VOXCPM2_MODELS),
            Self::VoxCpm2Q8 => Ok(catalog::VOXCPM2_Q8_MODELS),
            _ => Err("Model does not support VoxCPM2 synthesis".into()),
        }
    }
    pub(crate) fn from_alignment_spec(spec: &str) -> Result<Self, String> {
        [Self::Qwen3Aligner, Self::Qwen3AlignerQ8]
            .into_iter()
            .find(|model| model.alignment_spec().is_ok_and(|value| value == spec))
            .ok_or("runtime_mismatch: forced alignment model specification is unsupported".into())
    }
    pub(crate) fn alignment_revision(self) -> Result<&'static str, String> {
        match self {
            Self::Qwen3Aligner => Ok(catalog::QWEN3_FORCED_ALIGNER_REVISION),
            Self::Qwen3AlignerQ8 => Ok(catalog::QWEN3_ALIGNER_Q8_REVISION),
            _ => Err("Model does not support forced alignment".into()),
        }
    }
    pub(crate) fn diarization_variant(self) -> Result<catalog::DiarizationVariant, String> {
        match self {
            Self::SherpaDiarization => Ok(catalog::DiarizationVariant::Int8),
            Self::SherpaDiarizationFull => Ok(catalog::DiarizationVariant::Full),
            _ => Err("Model does not support speaker diarization".into()),
        }
    }
    pub(crate) fn from_diarization_spec(spec: &str) -> Result<Self, String> {
        MODELS
            .iter()
            .copied()
            .find(|model| {
                model
                    .diarization_variant()
                    .is_ok_and(|variant| variant.spec_id() == spec)
            })
            .ok_or(
                "runtime_mismatch: speaker diarization model specification is unsupported".into(),
            )
    }
    pub(crate) fn image_segmentation_spec(self) -> Result<&'static str, String> {
        match self {
            Self::U2netp => Ok(IMAGE_SEGMENTATION_SPEC_ID),
            Self::U2net => Ok(IMAGE_SEGMENTATION_QUALITY_SPEC_ID),
            _ => Err("Model does not support image segmentation".into()),
        }
    }
    pub(crate) fn from_separation_spec(spec: &str) -> Result<Self, String> {
        MODELS
            .iter()
            .copied()
            .find(|model| model.separation_spec().is_ok_and(|value| value == spec))
            .ok_or("runtime_mismatch: source separation model specification is unsupported".into())
    }
    pub(crate) fn separation_spec(self) -> Result<&'static str, String> {
        match self {
            Self::Spleeter2Stems => Ok("spleeter-2stems-fp16-cpu-v1"),
            Self::Spleeter2StemsFull => Ok("spleeter-2stems-fp32-cpu-v1"),
            _ => Err("Model does not support source separation".into()),
        }
    }
    pub(crate) fn is_essential(self, transcription: Self) -> bool {
        self == Self::Qwen3Embedding || self == transcription
    }
    pub(crate) fn is_transcription(self) -> bool {
        matches!(
            self,
            Self::WhisperSmall | Self::WhisperTiny | Self::WhisperLargeV3
        )
    }
    pub(crate) fn transcription_models(self) -> &'static [catalog::ModelKind] {
        if self == Self::WhisperSmall {
            catalog::TRANSCRIPTION_MODELS
        } else {
            transcription_snapshot(self).models()
        }
    }
}

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
