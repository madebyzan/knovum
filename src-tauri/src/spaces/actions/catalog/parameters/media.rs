use std::sync::OnceLock;

use schemars::{schema_for, JsonSchema};
use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum ImageAspectRatio {
    #[serde(rename = "1:1")]
    Square,
    #[serde(rename = "3:2")]
    Landscape,
    #[serde(rename = "2:3")]
    Portrait,
    #[serde(rename = "16:9")]
    Widescreen,
    #[serde(rename = "9:16")]
    Vertical,
}

impl ImageAspectRatio {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Square => "1:1",
            Self::Landscape => "3:2",
            Self::Portrait => "2:3",
            Self::Widescreen => "16:9",
            Self::Vertical => "9:16",
        }
    }
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ImageParameters {
    #[schemars(length(min = 1, max = 512))]
    title: String,
    #[schemars(length(min = 1))]
    image_style: String,
    aspect_ratio: ImageAspectRatio,
}

#[derive(Clone, Copy, Debug, Deserialize, JsonSchema, ts_rs::TS)]
#[ts(export_to = "../../../features/spaces/contracts/generated/")]
pub(crate) enum VideoAspectRatio {
    #[serde(rename = "1:1")]
    Square,
    #[serde(rename = "16:9")]
    Widescreen,
    #[serde(rename = "9:16")]
    Vertical,
}

impl VideoAspectRatio {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Square => "1:1",
            Self::Widescreen => "16:9",
            Self::Vertical => "9:16",
        }
    }
}

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct VideoParameters {
    #[schemars(length(min = 1, max = 512))]
    title: String,
    #[schemars(length(min = 1))]
    prompt: String,
    aspect_ratio: VideoAspectRatio,
    #[schemars(range(min = 5, max = 15))]
    duration_seconds: u16,
}

pub(crate) fn image() -> &'static Value {
    static SCHEMA: OnceLock<Value> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        serde_json::to_value(schema_for!(ImageParameters))
            .expect("Space Action schema must serialize")
    })
}

pub(crate) fn video() -> &'static Value {
    static SCHEMA: OnceLock<Value> = OnceLock::new();
    SCHEMA.get_or_init(|| {
        let mut schema = serde_json::to_value(schema_for!(VideoParameters))
            .expect("Space Action schema must serialize");
        schema["properties"]["durationSeconds"]["enum"] = serde_json::json!([5, 10, 15]);
        schema
    })
}
