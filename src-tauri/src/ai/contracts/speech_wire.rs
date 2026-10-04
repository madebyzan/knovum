use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InferenceRequestIdentity {
    pub request_id: String,
    pub owner_kind: String,
    pub owner_id: String,
    pub attempt_id: String,
    pub attempt_epoch: u64,
    pub input_revision: String,
    pub model_spec_id: String,
    pub deadline_ms: u64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AudioFileReference {
    pub path: String,
    pub media_type: String,
    pub byte_length: u64,
    pub sha256: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub sample_format: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechFileReference {
    pub file_name: String,
    pub media_type: String,
    pub byte_length: u64,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SynthesizedAudioFileReference {
    pub file_name: String,
    pub media_type: String,
    pub byte_length: u64,
    pub sha256: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub sample_format: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechSynthesisChunkEvent {
    pub request_id: String,
    pub attempt_epoch: u64,
    pub seq: u64,
    pub granularity: SpeechSynthesisChunkGranularity,
    pub chunk_ref: SynthesizedAudioFileReference,
    pub sample_count: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, PartialEq, Eq, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum SpeechSynthesisChunkGranularity {
    ModelChunk,
    Sentence,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum LanguagePolicy {
    Detect,
    Fixed,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum AsrEngine {
    FasterWhisper,
    MlxWhisper,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AsrProfile {
    pub engine: AsrEngine,
    pub language_policy: LanguagePolicy,
    pub language: Option<String>,
    pub device: String,
    pub compute_type: String,
    pub beam_size: u8,
    pub vad_enabled: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechTranscribeParams {
    #[serde(flatten)]
    pub identity: InferenceRequestIdentity,
    pub audio_ref: AudioFileReference,
    pub profile: AsrProfile,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechWordTiming {
    pub text: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub probability: Option<f32>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechTranscriptSegment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
    pub words: Vec<SpeechWordTiming>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechProgressEvent {
    pub request_id: String,
    pub attempt_epoch: u64,
    pub seq: u64,
    pub event_type: SpeechProgressEventType,
    pub stage: String,
    pub progress: u8,
    pub language: Option<String>,
    pub partial: Option<SpeechTranscriptSegment>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum SpeechProgressEventType {
    Progress,
    PartialTranscript,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechTranscribeResult {
    pub request_id: String,
    pub attempt_epoch: u64,
    pub model_spec_id: String,
    pub transcript_ref: SpeechFileReference,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechTranscriptArtifact {
    pub language: String,
    pub duration_ms: u64,
    pub speech_duration_ms: u64,
    pub segments: Vec<SpeechTranscriptSegment>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlignmentTextSegment {
    pub segment_id: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlignmentProfile {
    pub language: String,
    pub device: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechAlignParams {
    #[serde(flatten)]
    pub identity: InferenceRequestIdentity,
    pub audio_ref: AudioFileReference,
    pub text_segments: Vec<AlignmentTextSegment>,
    pub profile: AlignmentProfile,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum AlignmentUnitKind {
    Word,
    Character,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AlignedSpeechUnit {
    pub segment_id: String,
    pub text: String,
    pub unit_kind: AlignmentUnitKind,
    pub start_ms: Option<u64>,
    pub end_ms: Option<u64>,
    pub score: Option<f32>,
    pub unaligned_reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechAlignResult {
    pub request_id: String,
    pub attempt_epoch: u64,
    pub model_spec_id: String,
    pub units: Vec<AlignedSpeechUnit>,
    pub coverage: f32,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeakerCountBounds {
    pub minimum: Option<u16>,
    pub maximum: Option<u16>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechDiarizeParams {
    #[serde(flatten)]
    pub identity: InferenceRequestIdentity,
    pub audio_ref: AudioFileReference,
    pub speaker_count_bounds: SpeakerCountBounds,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeakerTurn {
    pub speaker_id: String,
    pub start_ms: u64,
    pub end_ms: u64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechDiarizeResult {
    pub request_id: String,
    pub attempt_epoch: u64,
    pub model_spec_id: String,
    pub turns: Vec<SpeakerTurn>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum VoiceSourceKind {
    Preset,
    Reference,
    Designed,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VoiceRevisionReference {
    pub revision_id: String,
    pub source_kind: VoiceSourceKind,
    pub speaker_id: Option<u32>,
    pub reference_audio: Option<AudioFileReference>,
    pub reference_text: Option<String>,
    pub design_prompt: Option<String>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum SpeechSynthesisEngine {
    Kokoro,
    VoxCpm2,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SynthesisProfile {
    pub engine: SpeechSynthesisEngine,
    pub language: String,
    pub speed: f32,
    pub sample_rate: u32,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechSynthesizeParams {
    #[serde(flatten)]
    pub identity: InferenceRequestIdentity,
    pub text: String,
    pub voice_revision: VoiceRevisionReference,
    pub style: Option<String>,
    pub seed: u64,
    pub profile: SynthesisProfile,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechSynthesizeResult {
    pub request_id: String,
    pub attempt_epoch: u64,
    pub model_spec_id: String,
    pub byte_length: u64,
    pub sha256: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub sample_format: String,
    pub sample_count: u64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechSeparateParams {
    #[serde(flatten)]
    pub identity: InferenceRequestIdentity,
    pub audio_ref: AudioFileReference,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechSeparateResult {
    pub request_id: String,
    pub attempt_epoch: u64,
    pub model_spec_id: String,
    pub vocals_ref: SynthesizedAudioFileReference,
    pub background_ref: SynthesizedAudioFileReference,
    pub sample_count: u64,
}
