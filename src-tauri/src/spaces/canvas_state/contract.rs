use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CanvasDocumentV1 {
    pub version: u8,
    pub viewport: CanvasViewport,
    pub nodes: Vec<CanvasNodeLayoutV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relation_styles: Option<Vec<CanvasRelationStyleV1>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CanvasRelationStyleV1 {
    pub relation_id: String,
    pub color: CanvasRelationColor,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasRelationColor {
    Blue,
    Green,
    Amber,
    Red,
    Purple,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanvasViewport {
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CanvasNodeLayoutV1 {
    pub node_id: String,
    pub frame: CanvasNodeFrame,
    pub size_authority: CanvasNodeSizeAuthority,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub appearance: Option<CanvasNodeAppearance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_subpath: Option<String>,
    pub z_index: u32,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasNodeSizeAuthority {
    Content,
    Frame,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CanvasNodeAppearance {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<CanvasFillPreset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_fill: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_color: Option<CanvasTextColor>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_size: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub typography_scale: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<CanvasFontWeight>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font_style: Option<CanvasFontStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_decoration: Option<CanvasTextDecoration>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_align: Option<CanvasTextAlign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vertical_align: Option<CanvasVerticalAlign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background_style: Option<CanvasBackgroundStyle>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasFillPreset {
    Yellow,
    Green,
    Blue,
    Pink,
    Gray,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasTextColor {
    Ink,
    Muted,
    Red,
    Blue,
    Green,
    Purple,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasFontWeight {
    Regular,
    Bold,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasFontStyle {
    Normal,
    Italic,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasTextDecoration {
    None,
    Underline,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasTextAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasVerticalAlign {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasBackgroundStyle {
    Cover,
    Ratio,
    Repeat,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CanvasNodeFrame {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StoredCanvasState {
    pub document: CanvasDocumentV1,
    pub revision: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct SaveCanvasStateRequest {
    pub space_id: String,
    pub base_revision: i64,
    pub document: CanvasDocumentV1,
}
