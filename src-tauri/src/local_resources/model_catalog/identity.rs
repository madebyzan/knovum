use super::{
    catalog, transcription_snapshot, LocalModelId, IMAGE_SEGMENTATION_HUMAN_SPEC_ID,
    IMAGE_SEGMENTATION_QUALITY_SPEC_ID, IMAGE_SEGMENTATION_SILUETA_SPEC_ID,
    IMAGE_SEGMENTATION_SPEC_ID, MODELS, VOXCPM2_BF16_SPEC_ID, VOXCPM2_FULL_SPEC_ID,
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
    pub(crate) const fn voice_variant(self) -> Option<catalog::VoiceVariant> {
        use catalog::VoiceVariant as V;
        match self {
            Self::IndexTts2 => Some(V::IndexTts2),
            Self::IndexTts2F16 => Some(V::IndexTts2F16),
            Self::Qwen3Tts => Some(V::Qwen3Tts),
            Self::Qwen3TtsBf16 => Some(V::Qwen3TtsBf16),
            Self::Qwen3TtsSmall => Some(V::Qwen3TtsSmall),
            Self::CosyVoice3 => Some(V::CosyVoice3),
            _ => None,
        }
    }
    /// The engine an audio.cpp voice model speaks with; MOSS-TTS-Nano is not one.
    pub(crate) const fn voice_engine(
        self,
    ) -> Option<crate::ai::contracts::speech_wire::SpeechSynthesisEngine> {
        use crate::ai::contracts::speech_wire::SpeechSynthesisEngine as E;
        match self {
            Self::VoxCpm2 | Self::VoxCpm2Bf16 | Self::VoxCpm2Q8 => Some(E::VoxCpm2),
            Self::IndexTts2 | Self::IndexTts2F16 => Some(E::IndexTts2),
            Self::Qwen3Tts | Self::Qwen3TtsBf16 | Self::Qwen3TtsSmall => Some(E::Qwen3Tts),
            Self::CosyVoice3 => Some(E::CosyVoice3),
            _ => None,
        }
    }
    /// The identity an audio.cpp voice model is stored and leased under.
    pub(crate) fn voice_spec(self) -> Result<&'static str, String> {
        match self.voice_variant() {
            Some(variant) => Ok(variant.spec_id()),
            None => self.voxcpm2_spec(),
        }
    }
    pub(crate) fn from_voice_spec(spec: &str) -> Result<Self, String> {
        MODELS
            .iter()
            .copied()
            .find(|model| model.voice_spec().is_ok_and(|value| value == spec))
            .ok_or("runtime_mismatch: voice model specification is unsupported".into())
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
    pub(crate) fn image_segmentation_spec(self) -> Result<&'static str, String> {
        match self {
            Self::U2netp => Ok(IMAGE_SEGMENTATION_SPEC_ID),
            Self::Silueta => Ok(IMAGE_SEGMENTATION_SILUETA_SPEC_ID),
            Self::U2netHumanSeg => Ok(IMAGE_SEGMENTATION_HUMAN_SPEC_ID),
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
    pub(crate) const fn separation(self) -> Option<catalog::SeparationVariant> {
        match self {
            Self::HtDemucs => Some(catalog::SeparationVariant::HtDemucs),
            Self::HtDemucsF16 => Some(catalog::SeparationVariant::HtDemucsF16),
            _ => None,
        }
    }
    pub(crate) fn separation_spec(self) -> Result<&'static str, String> {
        self.separation()
            .map(catalog::SeparationVariant::spec_id)
            .ok_or_else(|| "Model does not support source separation".into())
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
