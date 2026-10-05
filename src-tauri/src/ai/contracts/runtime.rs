use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u32 = 1;
pub const MAX_MESSAGE_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeCapability {
    Embedding,
    SpeechTranscription,
    SpeechAlignment,
    SpeechDiarization,
    SpeechSynthesis,
    SpeechSeparation,
    ImageSegmentation,
}

impl RuntimeCapability {
    pub fn method(self) -> &'static str {
        match self {
            Self::Embedding => "embedding.encode",
            Self::SpeechTranscription => "speech.transcribe",
            Self::SpeechAlignment => "speech.align",
            Self::SpeechDiarization => "speech.diarize",
            Self::SpeechSynthesis => "speech.synthesize",
            Self::SpeechSeparation => "speech.separate",
            Self::ImageSegmentation => "image.segment",
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeDeviceProfile {
    pub device: String,
    pub execution_profile_id: String,
    pub cpu_threads: u8,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeModelPaths {
    pub embedding: Option<String>,
    pub speech_transcription: Option<String>,
    pub speech_alignment: Option<String>,
    pub speech_diarization: Option<String>,
    pub speech_synthesis: Option<String>,
    pub speech_separation: Option<String>,
    pub image_segmentation: Option<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeInitializeParams {
    pub protocol_version: u32,
    pub runtime_digest: String,
    pub session_id: String,
    pub capability: RuntimeCapability,
    pub capability_spec_id: String,
    pub device_profile: RuntimeDeviceProfile,
    pub work_directory: String,
    pub model_paths: RuntimeModelPaths,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeLimits {
    pub max_message_bytes: u32,
    pub max_embedding_texts: u8,
    pub max_embedding_tokens: u32,
    pub max_unconsumed_chunk_files: u8,
    pub max_unconsumed_chunk_bytes: u32,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeSession {
    pub protocol_version: u32,
    pub runtime_digest: String,
    pub session_id: String,
    pub capability: RuntimeCapability,
    pub capability_spec_id: String,
    pub device_profile: RuntimeDeviceProfile,
    pub capabilities: Vec<String>,
    pub limits: RuntimeLimits,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeHealthParams {
    pub session_id: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeHealthObservation {
    pub session_id: String,
    pub current_request_id: Option<String>,
    pub resident_bytes: u64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeCancelParams {
    pub target_request_id: String,
    pub attempt_epoch: u64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeCancellationAcknowledgement {
    pub accepted: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeAckChunkParams {
    pub request_id: String,
    pub attempt_epoch: u64,
    pub seq: u64,
    pub consumed_bytes: u64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeShutdownParams {
    pub session_id: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeShutdownAcknowledgement {
    pub accepted: bool,
}
