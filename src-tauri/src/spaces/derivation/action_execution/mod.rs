mod document;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{Map, Value};
use sqlx::{Executor, Sqlite};
use tauri::{ipc::Channel, AppHandle};

use super::{grounded_document, subtitled_video, video_generation, visualization};
use crate::app_error::AppError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Handler {
    Answer,
    Text,
    Image,
    Video,
    Subtitles,
    Document,
}

fn handler(action_id: &str) -> Option<Handler> {
    match action_id {
        "answer-with-sources" => Some(Handler::Answer),
        "draft-document" | "draft-script" | "summarize-meeting" | "draft-social-post" => {
            Some(Handler::Text)
        }
        "generate-visualization" | "generate-image" => Some(Handler::Image),
        "generate-video" => Some(Handler::Video),
        "add-video-subtitles" => Some(Handler::Subtitles),
        "render-presentation"
        | "export-document-as-docx"
        | "export-document-as-pdf"
        | "export-document-as-txt" => Some(Handler::Document),
        _ => None,
    }
}

pub(crate) async fn execute(
    app: &AppHandle,
    action_id: &str,
    space_id: &str,
    parameters: &Value,
    inputs: &Value,
    authority: ExecutionAuthority,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<Value, AppError> {
    let prepared =
        super::action_request::prepare(app, action_id, space_id, parameters, inputs, &authority)
            .await?;
    let request = request_value(&authority.run_id, space_id, parameters, &prepared)?;
    match handler(action_id) {
        Some(Handler::Answer) => {
            super::grounded_answer::answer(app, decode(request)?, &authority, cancellation).await
        }
        Some(Handler::Text) => Ok(serialize(
            grounded_document::generate_for_action(
                authority.clone(),
                app.clone(),
                decode(text_request(action_id, request)?)?,
                quiet_channel(),
                cancellation,
            )
            .await?,
        )?),
        Some(Handler::Image) => Ok(serialize(
            visualization::generate_for_action(
                authority.clone(),
                app.clone(),
                decode(image_request(request)?)?,
                quiet_channel(),
                cancellation,
            )
            .await?,
        )?),
        Some(Handler::Video) => Ok(serialize(
            video_generation::generate_for_action(
                authority.clone(),
                app.clone(),
                decode(request)?,
                quiet_channel(),
                cancellation,
            )
            .await?,
        )?),
        Some(Handler::Subtitles) => Ok(serialize(
            subtitled_video::render_for_action(
                authority.clone(),
                app.clone(),
                decode(request)?,
                quiet_channel(),
                cancellation,
            )
            .await?,
        )?),
        Some(Handler::Document) => {
            document::execute(app, action_id, request, &authority, cancellation).await
        }
        None => Err(AppError::new(
            "action_executor_unavailable",
            format!("Durable Action {action_id} has no registered handler"),
        )),
    }
}

#[derive(Clone, Debug)]
pub(crate) struct ExecutionAuthority {
    pub run_id: String,
    pub execution_epoch: i64,
}

impl ExecutionAuthority {
    pub(super) async fn require_active<'e, E>(&self, executor: E) -> Result<(), String>
    where
        E: Executor<'e, Database = Sqlite>,
    {
        require_active_execution(executor, &self.run_id, self.execution_epoch).await
    }
}

pub(super) async fn require_active_execution<'e, E>(
    executor: E,
    run_id: &str,
    execution_epoch: i64,
) -> Result<(), String>
where
    E: Executor<'e, Database = Sqlite>,
{
    let active: Option<i64> = sqlx::query_scalar(
        "SELECT 1 FROM spaces_runs WHERE id=$1 AND execution_epoch=$2 AND state='running'
         AND cancel_requested_at IS NULL AND completed_at IS NULL",
    )
    .bind(run_id)
    .bind(execution_epoch)
    .fetch_optional(executor)
    .await
    .map_err(crate::spaces::db::sql_error)?;
    active
        .map(|_| ())
        .ok_or_else(|| "action-execution-stale: Run was cancelled, completed, or superseded".into())
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
        if key != "title" {
            request.insert(key.clone(), value.clone());
        }
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
    if action_id == "draft-social-post" {
        for key in ["platform", "audience", "goal", "tone"] {
            values.remove(key);
        }
    }
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

#[cfg(test)]
mod tests;
