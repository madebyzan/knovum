use std::sync::OnceLock;

use schemars::{schema_for, JsonSchema};
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, JsonSchema)]
pub(super) enum ImageAspectRatio {
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

#[derive(Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ImageParameters {
    #[schemars(length(min = 1, max = 512))]
    title: String,
    #[schemars(length(min = 1))]
    image_style: String,
    aspect_ratio: ImageAspectRatio,
}

#[derive(Deserialize, JsonSchema)]
enum VideoAspectRatio {
    #[serde(rename = "1:1")]
    Square,
    #[serde(rename = "16:9")]
    Widescreen,
    #[serde(rename = "9:16")]
    Vertical,
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
