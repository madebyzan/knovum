use serde::{Deserialize, Serialize};

use super::super::contracts::SpeechAudioOutput;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum VoiceModel {
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
}

impl VoiceModel {
    const ALL: [Self; 10] = [
        Self::MossNano,
        Self::VoxCpm2,
        Self::VoxCpm2Bf16,
        Self::VoxCpm2Q8,
        Self::IndexTts2,
        Self::IndexTts2F16,
        Self::Qwen3Tts,
        Self::Qwen3TtsBf16,
        Self::Qwen3TtsSmall,
        Self::CosyVoice3,
    ];

    pub(crate) fn local_model_id(self) -> crate::local_resources::model_catalog::LocalModelId {
        use crate::local_resources::model_catalog::LocalModelId as M;
        match self {
            Self::MossNano => M::MossNano,
            Self::VoxCpm2 => M::VoxCpm2,
            Self::VoxCpm2Bf16 => M::VoxCpm2Bf16,
            Self::VoxCpm2Q8 => M::VoxCpm2Q8,
            Self::IndexTts2 => M::IndexTts2,
            Self::IndexTts2F16 => M::IndexTts2F16,
            Self::Qwen3Tts => M::Qwen3Tts,
            Self::Qwen3TtsBf16 => M::Qwen3TtsBf16,
            Self::Qwen3TtsSmall => M::Qwen3TtsSmall,
            Self::CosyVoice3 => M::CosyVoice3,
        }
    }
    pub(crate) fn wire_engine(self) -> crate::ai::contracts::speech_wire::SpeechSynthesisEngine {
        self.local_model_id()
            .voice_engine()
            .unwrap_or(crate::ai::contracts::speech_wire::SpeechSynthesisEngine::MossNano)
    }
    pub(crate) fn engine(self) -> &'static str {
        self.wire_engine().id()
    }
    pub(crate) fn specification(self) -> &'static str {
        match self {
            Self::MossNano => super::super::execution::MOSS_NANO_MODEL_SPEC_ID,
            _ => self
                .local_model_id()
                .voice_spec()
                .expect("every audio.cpp voice model has a specification"),
        }
    }
    pub(crate) fn reference_duration(
        self,
    ) -> crate::local_resources::model_catalog::LocalModelReferenceDuration {
        crate::local_resources::model_metadata::metadata(self.local_model_id())
            .reference_duration
            .expect("Synthesis models declare reference audio duration bounds")
    }
    /// Only VoxCPM2 designs a voice from a description.
    pub(crate) fn supports_design(self) -> bool {
        self.wire_engine() == crate::ai::contracts::speech_wire::SpeechSynthesisEngine::VoxCpm2
    }
    /// Whether a confirmed transcript of the reference conditions the clone. MOSS-TTS-Nano and
    /// IndexTTS2 clone from the recording alone.
    pub(crate) fn accepts_transcript(self) -> bool {
        use crate::ai::contracts::speech_wire::SpeechSynthesisEngine as E;
        !matches!(self.wire_engine(), E::MossNano | E::IndexTts2)
    }
    pub(crate) fn from_specification(value: &str) -> Result<Self, String> {
        Self::ALL
            .into_iter()
            .find(|model| model.specification() == value)
            .ok_or_else(|| "Voice model specification is unsupported".into())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ts_rs::TS)]
#[serde(
    tag = "kind",
    rename_all = "snake_case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum VoiceCandidateSource {
    Reference {
        source_path: String,
        source_revision: String,
        start_ms: u64,
        end_ms: u64,
        reference_text: Option<String>,
        reference_text_confirmed: bool,
        #[serde(default)]
        design_prompt: Option<String>,
        /// Clone the voice alone: the selection's vocals are separated from music and effects
        /// before they become the reference, so the clone does not carry the soundtrack.
        #[serde(default)]
        isolate_voice: bool,
    },
    Designed {
        description: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct DraftVoiceCandidateRequest {
    pub model: VoiceModel,
    pub request_id: String,
    pub language: String,
    pub text: String,
    pub seed: u32,
    pub source: VoiceCandidateSource,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum VoiceCandidateStatus {
    Ready,
    Selected,
}

#[derive(Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct VoiceCandidate {
    pub model: VoiceModel,
    pub candidate_id: String,
    pub run_id: String,
    pub source_kind: String,
    pub language: String,
    pub sample_text: String,
    pub seed: u32,
    pub design_description: Option<String>,
    pub reference_text: Option<String>,
    pub audio: SpeechAudioOutput,
    pub status: VoiceCandidateStatus,
    pub auditioned_at: Option<i64>,
    pub selected_voice_profile_id: Option<String>,
    pub selected_voice_revision_id: Option<String>,
    pub selected_at: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct CreateCandidateVoiceRequest {
    pub candidate_id: String,
    pub name: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct VoiceCandidateStepConfiguration {
    pub request: DraftVoiceCandidateRequest,
    pub reference_source: Option<ReferenceSourceSnapshot>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct ReferenceSourceSnapshot {
    pub asset_path: String,
    pub byte_length: u64,
    pub source_revision: String,
    pub duration_ms: u64,
}
