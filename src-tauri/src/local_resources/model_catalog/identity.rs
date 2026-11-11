use super::{
    catalog::{self, SegmentationModel},
    transcription_snapshot, LocalModelId, MODELS, VOXCPM2_BF16_SPEC_ID, VOXCPM2_FULL_SPEC_ID,
    VOXCPM2_Q8_SPEC_ID,
};

impl LocalModelId {
    pub(crate) fn alignment_spec(self) -> Result<&'static str, String> {
        match self {
            Self::Qwen3Aligner => Ok("qwen3-forced-aligner-0.6b-f16-bundled-v2"),
            Self::Qwen3AlignerQ8 => Ok("qwen3-forced-aligner-0.6b-q8_0-v1"),
            _ => Err("Model does not support forced alignment".into()),
        }
    }
    pub(crate) fn voxcpm2_spec(self) -> Result<&'static str, String> {
        match self {
            Self::VoxCpm2 => Ok(VOXCPM2_FULL_SPEC_ID),
            Self::VoxCpm2Bf16 => Ok(VOXCPM2_BF16_SPEC_ID),
            Self::VoxCpm2Q8 => Ok(VOXCPM2_Q8_SPEC_ID),
            _ => Err("Model does not support VoxCPM2 synthesis".into()),
        }
    }
    pub(crate) fn from_voxcpm2_spec(spec: &str) -> Result<Self, String> {
        [Self::VoxCpm2, Self::VoxCpm2Bf16, Self::VoxCpm2Q8]
            .into_iter()
            .find(|model| model.voxcpm2_spec().is_ok_and(|value| value == spec))
            .ok_or("runtime_mismatch: VoxCPM2 model specification is unsupported".into())
    }
    pub(crate) fn voxcpm2_models(self) -> Result<&'static [catalog::ModelKind], String> {
        match self {
            Self::VoxCpm2 => Ok(catalog::VOXCPM2_MODELS),
            Self::VoxCpm2Bf16 => Ok(catalog::VOXCPM2_BF16_MODELS),
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
            Self::SherpaDiarizationCampplus => Ok(catalog::DiarizationVariant::Campplus),
            Self::SherpaDiarizationFull => Ok(catalog::DiarizationVariant::Full),
            Self::SherpaDiarizationEres2netLarge => Ok(catalog::DiarizationVariant::Eres2netLarge),
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
    /// The background-removal model this choice runs, if it is one.
    pub(crate) const fn segmentation(self) -> Option<SegmentationModel> {
        Some(match self {
            Self::BirefnetLite => SegmentationModel::BirefnetLite,
            Self::BirefnetGeneral => SegmentationModel::BirefnetGeneral,
            Self::BirefnetPortrait => SegmentationModel::BirefnetPortrait,
            Self::Ben2Base => SegmentationModel::Ben2Base,
            Self::IsnetAnime => SegmentationModel::IsnetAnime,
            _ => return None,
        })
    }
    pub(crate) fn image_segmentation_spec(self) -> Result<&'static str, String> {
        self.segmentation()
            .map(|model| model.spec().spec_id)
            .ok_or_else(|| "Model does not support image segmentation".into())
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
            Self::WhisperSmall
                | Self::WhisperTiny
                | Self::WhisperBase
                | Self::WhisperMedium
                | Self::WhisperMediumFull
                | Self::WhisperLargeV3TurboQ8
                | Self::WhisperLargeV3Turbo
                | Self::WhisperLargeV3
        )
    }
    pub(crate) fn transcription_models(self) -> &'static [catalog::ModelKind] {
        transcription_snapshot(self).models()
    }
}
