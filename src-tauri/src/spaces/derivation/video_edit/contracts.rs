use serde::{Deserialize, Serialize};

/// Absolute-time video edit project. Clips carry their own timeline placement, transitions are
/// anchored to a cut between two overlapping clips, and gaps expose the project background.
#[derive(Clone, PartialEq, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoEditProject {
    pub version: u8,
    pub source_revision: String,
    pub assets: Vec<VideoAsset>,
    pub output: VideoEditOutput,
    pub video_track: VideoTrack,
    pub transitions: Vec<VideoTransition>,
    pub audio_track: VideoEditAudioTrack,
    pub text_track: TextTrack,
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
    pub source_in_ms: u64,
    pub source_out_ms: u64,
    pub timeline_start_ms: u64,
    pub transform: VideoTransform,
    pub volume: f64,
    pub fade_in_ms: u64,
    pub fade_out_ms: u64,
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
    pub x: f64,
    pub y: f64,
    pub style: TextStyle,
}

#[derive(Clone, PartialEq, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TextStyle {
    pub family: String,
    pub weight: u16,
    pub italic: bool,
    pub size: u16,
    pub color: String,
    pub align: TextAlign,
    pub decoration: TextDecoration,
    pub decoration_strength: u16,
    pub decoration_color: String,
}

#[derive(Clone, PartialEq, Copy, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, PartialEq, Copy, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub enum TextDecoration {
    Plain,
    Outline,
    Shadow,
    Box,
}

#[derive(Clone, PartialEq, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VideoTransform {
    pub x: f64,
    pub y: f64,
    pub scale: f64,
    pub rotation: f64,
    pub opacity: f64,
    pub fit: VideoFit,
    pub flip_horizontal: bool,
    pub flip_vertical: bool,
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
    pub background: String,
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
    pub id: String,
    pub from_clip_id: String,
    pub kind: VideoTransitionKind,
    pub duration_ms: u64,
}

#[derive(Clone, PartialEq, Copy, Deserialize, Serialize, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
#[serde(rename_all = "camelCase")]
pub enum VideoTransitionKind {
    Dissolve,
    FadeBlack,
    FadeWhite,
    WipeLeft,
    WipeRight,
    WipeUp,
    WipeDown,
    SlideLeft,
    SlideRight,
    SlideUp,
    SlideDown,
}

impl VideoTransitionKind {
    pub(super) fn xfade_name(self) -> &'static str {
        match self {
            Self::Dissolve => "fade",
            Self::FadeBlack => "fadeblack",
            Self::FadeWhite => "fadewhite",
            Self::WipeLeft => "wipeleft",
            Self::WipeRight => "wiperight",
            Self::WipeUp => "wipeup",
            Self::WipeDown => "wipedown",
            Self::SlideLeft => "slideleft",
            Self::SlideRight => "slideright",
            Self::SlideUp => "slideup",
            Self::SlideDown => "slidedown",
        }
    }
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
    pub source_in_ms: u64,
    pub source_out_ms: u64,
    pub timeline_start_ms: u64,
    pub lane: u8,
    pub volume: f64,
    pub fade_in_ms: u64,
    pub fade_out_ms: u64,
}

impl VideoClip {
    pub(super) fn length_ms(&self) -> u64 {
        self.source_out_ms - self.source_in_ms
    }
    pub(super) fn timeline_end_ms(&self) -> u64 {
        self.timeline_start_ms + self.length_ms()
    }
}

impl VideoAudioClip {
    pub(super) fn length_ms(&self) -> u64 {
        self.source_out_ms - self.source_in_ms
    }
}
