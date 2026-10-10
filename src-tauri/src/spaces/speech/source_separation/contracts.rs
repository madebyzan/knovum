use serde::{Deserialize, Serialize};

use super::super::video_config::SpeechMediaSource;

#[derive(Debug, Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct StartSpeechSourceSeparationRequest {
    pub request_id: String,
    pub space_id: String,
    pub source_node_id: String,
    pub vocals_label: String,
    pub background_label: String,
    pub source: SpeechMediaSource,
}

#[derive(Clone, Debug, Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct SpeechSourceSeparationPublication {
    pub separation_id: String,
    pub space_id: String,
    pub run_id: String,
    pub source_node_id: String,
    pub source: SpeechMediaSource,
    pub background: SpeechMediaSource,
    pub vocals: SpeechMediaSource,
    pub model_spec_id: String,
}
