use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MediaTranscript {
    pub(crate) language: String,
    pub(crate) duration_ms: u64,
    pub(crate) text: String,
    pub(crate) segments: Vec<TranscriptSegment>,
    pub(crate) subtitle_cues: Vec<SubtitleCue>,
    pub(crate) waveform: Vec<f32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SubtitleCue {
    pub(crate) start_ms: u64,
    pub(crate) end_ms: u64,
    pub(crate) text: String,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TranscriptSegment {
    pub(crate) start_ms: u64,
    pub(crate) end_ms: u64,
    pub(crate) text: String,
}
