use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const VOICE_REFERENCE_SAMPLE_RATE: u32 = 16_000;
pub const VOICE_REFERENCE_MIN_DURATION_MS: u64 = 5_000;
pub const DESIGNED_VOICE_REFERENCE_MIN_DURATION_MS: u64 = 1_000;
pub const VOXCPM2_MAX_REFERENCE_DURATION_MS: u64 = 15_000;
pub const MOSS_NANO_MAX_REFERENCE_DURATION_MS: u64 = 30_000;
/// The longest reference the audio.cpp cloning families (IndexTTS2, Qwen3-TTS, CosyVoice3) take;
/// each conditions on a short prompt and degrades, not improves, past it.
pub const CLONING_MAX_REFERENCE_DURATION_MS: u64 = 15_000;
/// The most an engine that controls duration may shorten a line by generating it shorter. Below
/// this the speech is audibly rushed in the model itself; the rest is left to time compression.
pub const MIN_DURATION_SCALE: f32 = 0.75;
pub const VOICE_REFERENCE_FRAME_TOLERANCE_SAMPLES: u32 = 16;
pub const MAX_SYNTHESIS_TEXT_CHARACTERS: usize = 2_000;

/// Source separation reads and writes 16-bit stereo PCM at this rate.
pub const SEPARATION_SAMPLE_RATE: u32 = 44_100;
/// The longest source a separation accepts: six hours, the most a stereo 16-bit WAV stem can hold
/// at [`SEPARATION_SAMPLE_RATE`] within the format's 32-bit data size.
pub const SEPARATION_MAX_DURATION_MS: u64 = 6 * 60 * 60 * 1_000;

pub const fn voice_reference_sample_bounds(
    min_duration_ms: u64,
    max_duration_ms: u64,
) -> (u32, u32) {
    let rate = VOICE_REFERENCE_SAMPLE_RATE as u64;
    let tolerance = VOICE_REFERENCE_FRAME_TOLERANCE_SAMPLES as u64;
    let min_samples = (min_duration_ms * rate / 1_000).saturating_sub(tolerance) as u32;
    let max_samples = (max_duration_ms * rate / 1_000 + tolerance) as u32;
    (min_samples, max_samples)
}

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
    /// Milliseconds of execution the worker may spend on this request, measured from the moment
    /// the worker admits it. A budget is relative because only the executor knows when the work
    /// started: time spent queueing for the inference slot belongs to the host, not to the worker.
    pub budget_ms: u64,
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
    WhisperCpp,
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
pub struct SpeechTranscription {
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
pub struct SpeechAlignment {
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
pub struct SpeechDiarization {
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

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum SpeechSynthesisEngine {
    MossNano,
    VoxCpm2,
    IndexTts2,
    Qwen3Tts,
    CosyVoice3,
}

impl SpeechSynthesisEngine {
    pub const ALL: [Self; 5] = [
        Self::MossNano,
        Self::VoxCpm2,
        Self::IndexTts2,
        Self::Qwen3Tts,
        Self::CosyVoice3,
    ];

    /// The identity a voice revision stores (`spaces_voice_revisions.engine_id`).
    pub const fn id(self) -> &'static str {
        match self {
            Self::MossNano => "moss_nano",
            Self::VoxCpm2 => "voxcpm2",
            Self::IndexTts2 => "index_tts2",
            Self::Qwen3Tts => "qwen3_tts",
            Self::CosyVoice3 => "cosyvoice3",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|engine| engine.id() == id)
    }

    /// Whether a line's delivery instruction reaches the engine. Qwen3-TTS's cloning model has no
    /// style input, and MOSS-TTS-Nano none at all.
    pub const fn accepts_style(self) -> bool {
        matches!(self, Self::VoxCpm2 | Self::IndexTts2 | Self::CosyVoice3)
    }

    /// Whether the engine generates a line to a requested length (`duration_scale`), rather than
    /// only at its own pace.
    pub const fn controls_duration(self) -> bool {
        matches!(self, Self::IndexTts2)
    }

    pub const fn max_reference_duration_ms(self) -> u64 {
        match self {
            Self::MossNano => MOSS_NANO_MAX_REFERENCE_DURATION_MS,
            Self::VoxCpm2 => VOXCPM2_MAX_REFERENCE_DURATION_MS,
            Self::IndexTts2 | Self::Qwen3Tts | Self::CosyVoice3 => {
                CLONING_MAX_REFERENCE_DURATION_MS
            }
        }
    }
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
    /// The share of its natural length the line is generated at, for an engine that
    /// [controls duration](SpeechSynthesisEngine::controls_duration); `None` speaks it naturally.
    pub duration_scale: Option<f32>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SpeechSynthesis {
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
pub struct SpeechSeparation {
    pub request_id: String,
    pub attempt_epoch: u64,
    pub model_spec_id: String,
    pub vocals_ref: SynthesizedAudioFileReference,
    pub background_ref: SynthesizedAudioFileReference,
    pub sample_count: u64,
}
