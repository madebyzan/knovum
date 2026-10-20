use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct CreateWanRequest<'a> {
    pub model: &'a str,
    pub input: WanInput<'a>,
    pub parameters: WanParameters<'a>,
}

#[derive(Serialize)]
pub struct WanInput<'a> {
    pub prompt: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub media: Vec<WanMedia<'a>>,
}

#[derive(Serialize)]
pub struct WanMedia<'a> {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub url: &'a str,
}

#[derive(Serialize)]
pub struct WanParameters<'a> {
    pub resolution: &'static str,
    pub ratio: &'a str,
    pub duration: u8,
    pub prompt_extend: bool,
}

#[derive(Deserialize)]
pub struct WanResponse {
    pub output: Option<WanOutput>,
    pub code: Option<String>,
    pub message: Option<String>,
}

#[derive(Deserialize)]
pub struct WanOutput {
    pub task_id: String,
    pub task_status: String,
    pub video_url: Option<String>,
    pub code: Option<String>,
    pub message: Option<String>,
}

pub fn create_wan_request<'a>(
    model: &'a str,
    prompt: &'a str,
    ratio: &'a str,
    duration_seconds: u8,
    references: &'a [String],
) -> CreateWanRequest<'a> {
    CreateWanRequest {
        model,
        input: WanInput {
            prompt,
            media: references
                .iter()
                .map(|url| WanMedia {
                    kind: "reference_image",
                    url,
                })
                .collect(),
        },
        parameters: WanParameters {
            resolution: "720P",
            ratio,
            duration: duration_seconds,
            prompt_extend: true,
        },
    }
}
