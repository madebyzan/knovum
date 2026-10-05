use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Deserialize, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub enum VideoDeliveryAsset {
    Srt,
    Vtt,
    DialogueWav,
    BackgroundWav,
}

#[derive(Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct ExportVideoDubbingAssetRequest {
    pub job_id: String,
    pub project_id: String,
    pub revision_id: String,
    pub asset: VideoDeliveryAsset,
    pub destination_path: String,
}

#[derive(Serialize, ts_rs::TS)]
#[serde(rename_all = "camelCase")]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct VideoAudioTrack {
    pub revision_id: String,
    pub revision: i64,
    pub language: String,
}

#[derive(Deserialize, ts_rs::TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub struct ExportVideoAudioTracksRequest {
    pub job_id: String,
    pub project_id: String,
    pub revision_ids: Vec<String>,
    pub retain_original_track: bool,
    pub destination_path: String,
}
