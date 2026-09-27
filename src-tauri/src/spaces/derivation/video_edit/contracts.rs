use serde::{Deserialize, Serialize};

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoEditProject {
    pub version: u8,
    pub source_revision: String,
    pub source_width: u32,
    pub source_height: u32,
    pub video_track: VideoTrack,
    pub text_track: TextTrack,
    pub transform: VideoTransform,
}

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoTrack {
    pub clips: Vec<VideoClip>,
}

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoClip {
    pub id: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub volume: f64,
}

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextTrack {
    pub items: Vec<TextOverlay>,
}

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextOverlay {
    pub id: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
    pub placement: TextPlacement,
    pub style: TextStyle,
}

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextStyle {
    pub family: String,
    pub weight: u16,
    pub size: u16,
    pub color: String,
    pub preset: TextPreset,
}

#[derive(Clone, Copy, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub enum TextPreset {
    Plain,
    Outline,
    Shadow,
    Badge,
}

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoTransform {
    pub rotation: u16,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
}

#[derive(Clone, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub enum TextPlacement {
    Top,
    Center,
    Bottom,
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RenderVideoEditRequest {
    pub id: String,
    pub space_id: String,
    pub source_node_id: String,
    pub output_node_id: String,
    pub source_revision: String,
    pub project: VideoEditProject,
}

#[derive(Clone, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub struct VideoEditProgress {
    pub(super) stage: &'static str,
    pub(super) progress: u8,
}

#[derive(Deserialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PrepareVideoEditProxyRequest {
    pub id: String,
    pub space_id: String,
    pub source_node_id: String,
    pub source_revision: String,
}
