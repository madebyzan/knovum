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
    ImageUpscaling,
    ImageInpainting,
    ImageSelection,
    ImageDepth,
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
            Self::ImageUpscaling => "image.upscale",
            Self::ImageInpainting => "image.inpaint",
            Self::ImageSelection => "image.select",
            Self::ImageDepth => "image.depth",
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

/// The verified snapshot a worker loads.
///
/// A worker freezes one capability at initialization, so it is handed exactly one model. The
/// capability names every graph that model contains, except for the image capabilities that admit
/// more than one frozen variant, which name the variant the host prepared.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RuntimeModel {
    Snapshot {
        path: String,
    },
    ImageVariant {
        path: String,
        variant: super::image_compute::ImageModelVariant,
    },
}

impl RuntimeModel {
    /// The path this worker loads, whichever shape its model takes.
    pub fn path(&self) -> &str {
        match self {
            Self::Snapshot { path } | Self::ImageVariant { path, .. } => path,
        }
    }

    /// The snapshot of a capability whose graphs it alone determines.
    pub fn snapshot(&self) -> Result<&str, String> {
        match self {
            Self::Snapshot { path } => Ok(path),
            Self::ImageVariant { .. } => {
                Err("runtime_mismatch: capability does not take an image variant".into())
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeInitializeParams {
    pub runtime_manifest_path: String,
    pub protocol_version: u32,
    pub runtime_digest: String,
    pub session_id: String,
    pub capability: RuntimeCapability,
    pub capability_spec_id: String,
    pub device_profile: RuntimeDeviceProfile,
    pub work_directory: String,
    pub model: RuntimeModel,
    /// Milliseconds the worker may spend reaching a usable state. The host derives it from the
    /// runtime it must verify and the weights it must load, and both sides honour this one value
    /// so that a slow cold start cannot be aborted by a budget only one side knows about.
    pub budget_ms: u64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RuntimeLimits {
    pub max_message_bytes: u32,
    pub max_embedding_texts: u8,
    pub max_embedding_tokens: u32,
    pub max_unconsumed_chunk_files: u8,
    pub max_unconsumed_chunk_bytes: u32,
}

impl RuntimeLimits {
    /// The limits this protocol version fixes.
    ///
    /// The worker announces these and the host requires them, both from here, so a revision
    /// cannot leave one side declaring a bound the other rejects.
    pub const CURRENT: Self = Self {
        max_message_bytes: MAX_MESSAGE_BYTES as u32,
        max_embedding_texts: 32,
        max_embedding_tokens: 2048,
        max_unconsumed_chunk_files: 8,
        max_unconsumed_chunk_bytes: 16 * 1024 * 1024,
    };
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

impl RuntimeSession {
    /// Establishes that the worker that answered is the worker that was asked for.
    ///
    /// A session the host did not describe, or one that announces a capability or bound this
    /// protocol version does not fix, is not usable by the request that started it.
    pub fn matches(&self, params: &RuntimeInitializeParams) -> bool {
        self.protocol_version == params.protocol_version
            && self.runtime_digest == params.runtime_digest
            && self.session_id == params.session_id
            && self.capability == params.capability
            && self.capability_spec_id == params.capability_spec_id
            && self.device_profile == params.device_profile
            && self.capabilities == [params.capability.method()]
            && self.limits == RuntimeLimits::CURRENT
    }
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
