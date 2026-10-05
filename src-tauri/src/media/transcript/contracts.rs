use serde::Serialize;

#[derive(serde::Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct MediaTranscript {
    pub(crate) id: String,
    pub(crate) source_revision: String,
    pub(crate) revision: u32,
    pub(crate) language: String,
    pub(crate) duration_ms: u64,
    pub(crate) text: String,
    pub(crate) segments: Vec<NormalizedTranscriptSegment>,
    pub(crate) subtitle_cues: Vec<SubtitleCue>,
    pub(crate) provenance: TranscriptProvenance,
    pub(crate) speaker_tracks: Vec<SpeakerTrack>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub(crate) alignment: Option<TranscriptAlignmentProvenance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub(crate) diarization: Option<TranscriptDiarizationProvenance>,
}

#[derive(serde::Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SubtitleCue {
    pub(crate) start_ms: u64,
    pub(crate) end_ms: u64,
    pub(crate) text: String,
}

#[derive(serde::Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct NormalizedTranscriptSegment {
    pub(crate) id: String,
    pub(crate) revision: u32,
    pub(crate) start_ms: u64,
    pub(crate) end_ms: u64,
    pub(crate) start_sample: u64,
    pub(crate) end_sample: u64,
    pub(crate) text: String,
    pub(crate) speaker_id: Option<String>,
    pub(crate) units: Vec<TranscriptUnit>,
}

#[derive(serde::Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct TranscriptUnit {
    pub(crate) id: String,
    pub(crate) text: String,
    pub(crate) start_character: u32,
    pub(crate) end_character: u32,
    pub(crate) start_ms: Option<u64>,
    pub(crate) end_ms: Option<u64>,
    pub(crate) score: Option<f32>,
    pub(crate) unaligned_reason: Option<String>,
    pub(crate) unit_kind: String,
}

#[derive(serde::Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct TranscriptProvenance {
    pub(crate) engine_id: String,
    pub(crate) model_spec_id: String,
    pub(crate) model_revision: String,
    pub(crate) timing: String,
}

#[derive(serde::Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct TranscriptAlignmentProvenance {
    pub(crate) engine_id: String,
    pub(crate) model_spec_id: String,
    pub(crate) model_revision: String,
    pub(crate) aligned_at: u64,
}

#[derive(serde::Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct TranscriptDiarizationProvenance {
    pub(crate) engine_id: String,
    pub(crate) model_spec_id: String,
    pub(crate) model_revision: String,
    pub(crate) diarized_at: u64,
}

#[derive(serde::Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct SpeakerTrack {
    pub(crate) speaker_id: String,
    pub(crate) turns: Vec<SpeakerTurn>,
}

#[derive(serde::Deserialize, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) struct SpeakerTurn {
    pub(crate) start_ms: u64,
    pub(crate) end_ms: u64,
}

#[derive(Clone, Serialize, serde::Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct TranscriptSegment {
    pub(crate) start_ms: u64,
    pub(crate) end_ms: u64,
    pub(crate) text: String,
}
