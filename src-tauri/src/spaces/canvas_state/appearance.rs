use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasNodeSizeAuthority {
    Content,
    Frame,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
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

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasFillPreset {
    Yellow,
    Green,
    Blue,
    Pink,
    Gray,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasTextColor {
    Ink,
    Muted,
    Red,
    Blue,
    Green,
    Purple,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasFontWeight {
    Regular,
    Bold,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasFontStyle {
    Normal,
    Italic,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasTextDecoration {
    None,
    Underline,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasTextAlign {
    Left,
    Center,
    Right,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasVerticalAlign {
    Top,
    Middle,
    Bottom,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum CanvasBackgroundStyle {
    Cover,
    Ratio,
    Repeat,
}
