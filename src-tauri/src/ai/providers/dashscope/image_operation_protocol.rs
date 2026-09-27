use serde::Deserialize;

use super::image_operation::SourceImageOperation;

#[derive(Deserialize)]
pub struct Response {
    pub output: Option<Output>,
    pub code: Option<String>,
    pub message: Option<String>,
}

#[derive(Deserialize)]
pub struct Output {
    pub task_id: Option<String>,
    pub task_status: Option<String>,
    pub results: Option<Vec<GeneratedImageLocation>>,
    pub choices: Option<Vec<Choice>>,
    pub code: Option<String>,
    pub message: Option<String>,
}

#[derive(Deserialize)]
pub struct GeneratedImageLocation {
    pub url: String,
}

#[derive(Deserialize)]
pub struct Choice {
    pub message: ChoiceMessage,
}

#[derive(Deserialize)]
pub struct ChoiceMessage {
    pub content: Vec<ChoiceContent>,
}

#[derive(Deserialize)]
pub struct ChoiceContent {
    pub image: Option<String>,
}

pub fn request(operation: SourceImageOperation<'_>) -> (&'static str, serde_json::Value) {
    match operation {
        SourceImageOperation::Upscale { data_uri, factor } => (
            "services/aigc/image2image/image-synthesis",
            serde_json::json!({
                "model": "wanx2.1-imageedit",
                "input": { "function": "super_resolution", "prompt": "Image super resolution", "base_image_url": data_uri },
                "parameters": { "n": 1, "watermark": false, "upscale_factor": factor }
            }),
        ),
        SourceImageOperation::View { data_uri, prompt } => (
            "services/aigc/image-generation/generation",
            serde_json::json!({
                "model": "qwen-image-3.0-pro",
                "input": { "messages": [{
                    "role": "user",
                    "content": [{ "image": data_uri }, { "text": prompt }]
                }]},
                "parameters": { "n": 1, "prompt_extend": true, "watermark": false }
            }),
        ),
    }
}

pub fn image_url(output: &Output) -> Option<&str> {
    if let Some(url) = output
        .results
        .as_ref()
        .and_then(|results| results.first())
        .map(|generated_image| generated_image.url.as_str())
    {
        return Some(url);
    }
    output
        .choices
        .as_ref()?
        .iter()
        .flat_map(|choice| &choice.message.content)
        .find_map(|content| content.image.as_deref())
}
