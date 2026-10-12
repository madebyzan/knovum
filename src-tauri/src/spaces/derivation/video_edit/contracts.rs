use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoEditProject {
    pub version: u8,
    pub source_revision: String,
    pub assets: Vec<VideoAsset>,
    pub output: VideoEditOutput,
    pub video_track: VideoTrack,
    pub text_track: TextTrack,
    pub audio_track: VideoEditAudioTrack,
}

#[derive(Clone, PartialEq, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoTrack {
    pub clips: Vec<VideoClip>,
}

#[derive(Clone, PartialEq, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoClip {
    pub id: String,
    pub asset_id: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub volume: f64,
    pub fade_in_ms: u64,
    pub fade_out_ms: u64,
    pub transform: VideoTransform,
    #[serde(deserialize_with = "Option::deserialize")]
    pub transition: Option<VideoTransition>,
}

#[derive(Clone, PartialEq, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextTrack {
    pub items: Vec<TextOverlay>,
}

#[derive(Clone, PartialEq, Deserialize, Serialize, ts_rs::TS)]
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

#[derive(Clone, PartialEq, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextStyle {
    pub family: String,
    pub weight: u16,
    pub size: u16,
    pub color: String,
    pub preset: TextPreset,
}

#[derive(Clone, PartialEq, Copy, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub enum TextPreset {
    Plain,
    Outline,
    Shadow,
    Badge,
}

#[derive(Clone, PartialEq, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoTransform {
    pub rotation: u16,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
    pub fit: VideoFit,
    pub zoom: f64,
}

#[derive(Clone, PartialEq, Deserialize, Serialize, ts_rs::TS)]
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

#[derive(Clone, PartialEq, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoAsset {
    pub node_id: String,
    pub source_revision: String,
    pub kind: VideoAssetKind,
    pub duration_ms: u64,
    pub width: u32,
    pub height: u32,
}
#[derive(Clone, Copy, PartialEq, Eq, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub enum VideoAssetKind {
    Video,
    Image,
    Audio,
}
#[derive(Clone, PartialEq, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoEditOutput {
    pub width: u32,
    pub height: u32,
    pub frame_rate: u16,
}
#[derive(Clone, PartialEq, Copy, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub enum VideoFit {
    Contain,
    Cover,
}
#[derive(Clone, PartialEq, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoTransition {
    pub kind: VideoTransitionKind,
    pub duration_ms: u64,
}
#[derive(Clone, PartialEq, Copy, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub enum VideoTransitionKind {
    Dissolve,
    WipeLeft,
    WipeRight,
}
#[derive(Clone, PartialEq, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoEditAudioTrack {
    pub clips: Vec<VideoAudioClip>,
}
#[derive(Clone, PartialEq, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoAudioClip {
    pub id: String,
    pub asset_id: String,
    pub start_ms: u64,
    pub end_ms: u64,
    pub timeline_start_ms: u64,
    pub volume: f64,
    pub fade_in_ms: u64,
    pub fade_out_ms: u64,
}
