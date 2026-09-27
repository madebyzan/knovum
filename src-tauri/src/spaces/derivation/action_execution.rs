use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};
use tauri::{ipc::Channel, AppHandle};

use super::{
    grounded_document, subtitle_extraction, subtitled_video, video_generation, visualization,
};

pub(crate) async fn execute(
    app: &AppHandle,
    action_id: &str,
    run_id: &str,
    space_id: &str,
    parameters: &Value,
    inputs: &Value,
) -> Result<Value, String> {
    let prepared =
        super::action_request::prepare(app, action_id, run_id, space_id, parameters, inputs)
            .await?;
    let request = request_value(run_id, space_id, parameters, &prepared)?;
    match action_id {
        "answer-with-sources" => super::grounded_answer::answer(app, decode(request)?).await,
        "draft-document" | "draft-script" | "summarize-meeting" | "draft-social-post" => serialize(
            grounded_document::generate(
                app.clone(),
                decode(text_request(action_id, request)?)?,
                quiet_channel(),
            )
            .await?,
        ),
        "generate-visualization" | "generate-image" => serialize(
            visualization::generate(
                app.clone(),
                decode(image_request(request)?)?,
                quiet_channel(),
            )
            .await?,
        ),
        "generate-video" => serialize(
            video_generation::generate(app.clone(), decode(request)?, quiet_channel()).await?,
        ),
        "extract-subtitles" => serialize(subtitle_extraction::create(app, decode(request)?).await?),
        "render-subtitled-video" => serialize(
            subtitled_video::render(app.clone(), decode(request)?, quiet_channel()).await?,
        ),
        "render-presentation" => render_presentation(app, request).await,
        _ => Err(format!(
            "action-executor-unavailable: durable Action {action_id} has no registered handler"
        )),
    }
}

async fn render_presentation(app: &AppHandle, request: Value) -> Result<Value, String> {
    let document = request
        .pointer("/document/data")
        .and_then(Value::as_object)
        .ok_or("action-input-invalid: presentation document data is required")?;
    let source = crate::documents::generation::document::DocumentSource {
        title: required_text(document, "title")?.to_owned(),
        markdown: required_text(document, "text")?.to_owned(),
        citations: vec![],
    };
    let bytes =
        crate::blocking::run(move || crate::documents::generation::pptx::render(&source)).await?;
    serialize(super::presentation_publication::publish(app, request, bytes).await?)
}

pub(crate) async fn cancel(action_id: &str, run_id: &str) -> Result<(), String> {
    match action_id {
        "draft-document"
        | "draft-script"
        | "summarize-meeting"
        | "draft-social-post"
        | "generate-visualization"
        | "generate-image" => crate::ai::executions::cancel(run_id),
        "generate-video" => video_generation::registry::cancel(run_id),
        "render-subtitled-video" => subtitled_video::cancel(run_id.to_owned()).await,
        _ => Ok(()),
    }
}

fn request_value(
    run_id: &str,
    space_id: &str,
    parameters: &Value,
    inputs: &Value,
) -> Result<Value, String> {
    let mut request = match inputs {
        Value::Object(values) => values.clone(),
        Value::Array(values) => Map::from_iter([("inputs".into(), Value::Array(values.clone()))]),
        Value::Null => Map::new(),
        _ => return Err("action-input-invalid: inputs must be an object or array".into()),
    };
    let values = parameters
        .as_object()
        .ok_or("action-input-invalid: parameters must be an object")?;
    for (key, value) in values {
        request.insert(key.clone(), value.clone());
    }
    request.insert("id".into(), run_id.into());
    request.insert("spaceId".into(), space_id.into());
    Ok(Value::Object(request))
}

fn text_request(action_id: &str, mut request: Value) -> Result<Value, String> {
    let values = request
        .as_object_mut()
        .ok_or("action-input-invalid: request must be an object")?;
    let purpose = match action_id {
        "draft-document" => values.remove("purpose"),
        "draft-script" => values.remove("prompt"),
        "summarize-meeting" => values.remove("summaryPurpose"),
        "draft-social-post" => Some(Value::String(format!(
            "Write social copy for platform={}, audience={}, goal={}, tone={}",
            required_text(values, "platform")?,
            required_text(values, "audience")?,
            required_text(values, "goal")?,
            required_text(values, "tone")?,
        ))),
        _ => None,
    }
    .ok_or("action-input-invalid: text generation purpose is required")?;
    values.insert("purpose".into(), purpose);
    Ok(request)
}

fn image_request(mut request: Value) -> Result<Value, String> {
    let values = request
        .as_object_mut()
        .ok_or("action-input-invalid: request must be an object")?;
    if let Some(style) = values.remove("imageStyle") {
        values.insert("style".into(), style);
    }
    if !values.contains_key("intent") {
        values.insert(
            "intent".into(),
            "Create a companion image for the selected document".into(),
        );
    }
    Ok(request)
}

fn required_text<'a>(values: &'a Map<String, Value>, key: &str) -> Result<&'a str, String> {
    values
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("action-input-invalid: {key} is required"))
}

fn decode<T: DeserializeOwned>(value: Value) -> Result<T, String> {
    serde_json::from_value(value).map_err(crate::spaces::db::json_error)
}

fn serialize<T: Serialize>(value: T) -> Result<Value, String> {
    serde_json::to_value(value).map_err(crate::spaces::db::json_error)
}

fn quiet_channel<T: Serialize + Send + Sync + 'static>() -> Channel<T> {
    Channel::new(|_| Ok(()))
}
