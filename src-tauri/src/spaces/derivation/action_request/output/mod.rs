use serde_json::{json, Map, Value};
use tauri::AppHandle;

use super::inputs::ResolvedInputs;
use crate::foundation::app_error::AppError;
use crate::spaces::actions::{SpaceActionInputRole, SpaceActionOutputArity};

mod image;
mod media;

pub(super) async fn reserve(
    app: &AppHandle,
    space_id: &str,
    action_id: &str,
    parameters: &Value,
    inputs: &ResolvedInputs,
    request: &mut Map<String, Value>,
    authority: &super::super::action_execution::ExecutionAuthority,
) -> Result<(), AppError> {
    let definition = crate::spaces::actions::require(action_id)?;
    if definition.input_roles == [SpaceActionInputRole::Image] {
        image::request(inputs, request)?;
    }
    match action_id {
        "add-video-subtitles" => media::subtitled_video_request(inputs, request)?,
        "render-presentation"
        | "convert-document-to-docx"
        | "convert-document-to-pdf"
        | "convert-document-to-txt"
        | "convert-document-to-html" => media::document_request(inputs, action_id, request)?,
        _ => {}
    }
    // A discovered-arity handler reserves its own outputs once it knows how many there are.
    if definition.output_arity == SpaceActionOutputArity::Discovered {
        return Ok(());
    }
    let Some((kind, data)) = build_pending_output_content(app, action_id, parameters, inputs)?
    else {
        return Ok(());
    };
    let node_id = format!("{}-output", authority.run_id);
    request.insert("outputNodeId".into(), node_id.clone().into());
    if super::super::pending_output::exists(app, space_id, &node_id).await? {
        return Ok(());
    }
    super::super::pending_output::reserve(
        app,
        space_id,
        vec![super::super::pending_output::PendingOutput {
            node_id,
            kind,
            data,
        }],
        &inputs
            .nodes
            .iter()
            .map(|node| node.id.clone())
            .collect::<Vec<_>>(),
        authority,
    )
    .await
}

fn build_pending_output_content(
    app: &AppHandle,
    action_id: &str,
    parameters: &Value,
    inputs: &ResolvedInputs,
) -> Result<Option<(&'static str, Value)>, AppError> {
    let title = title(parameters, inputs)?;
    if crate::spaces::actions::is_image_transform(action_id) {
        return Ok(Some((
            "image",
            json!({"kind":"image","title":title,"fileName":format!("{title}.png"),"assetPath":super::super::output_paths::reserve(app,"generated","png")?,"processingState":"processing"}),
        )));
    }
    let output = match action_id {
        "draft-document" | "draft-script" | "summarize-meeting" | "draft-social-post" => (
            "markdown",
            json!({"kind":"markdown","title":title,"text":"","derivationState":"pending"}),
        ),
        "generate-visualization" | "generate-image" => (
            "image",
            json!({
                "kind":"image","title":title,"fileName":format!("{title}.png"),
                "assetPath": super::super::output_paths::reserve(app, "generated", "png")?,
                "processingState":"processing"
            }),
        ),
        "generate-video" => (
            "video",
            json!({
                "kind":"video","title":title,"fileName":format!("{title}.mp4"),
                "assetPath": super::super::output_paths::reserve(app, "generated", "mp4")?,
                "processingState":"processing"
            }),
        ),
        "add-video-subtitles" => ("video", media::build_subtitled_video_content(app, &title)?),
        "render-presentation"
        | "convert-document-to-docx"
        | "convert-document-to-pdf"
        | "convert-document-to-txt"
        | "convert-document-to-html" => {
            let target = super::super::document_formats::require(action_id)?;
            let document = inputs.nodes.first().ok_or("Document source is required")?;
            let title = super::super::document_formats::resolve_source_title(
                &document.kind,
                &document.data,
            )?;
            (
                "file",
                json!({"kind":"file", "title":title, "fileName":format!("{title}.{}", target.extension), "assetPath":super::super::output_paths::reserve(app, "document", target.extension)?, "processingState":"processing"}),
            )
        }
        _ => return Ok(None),
    };
    Ok(Some(output))
}

fn title(parameters: &Value, inputs: &ResolvedInputs) -> Result<String, AppError> {
    parameters
        .get("title")
        .and_then(Value::as_str)
        .or_else(|| {
            inputs
                .nodes
                .first()
                .and_then(|node| node.data.get("title"))
                .and_then(Value::as_str)
        })
        .filter(|title| !title.trim().is_empty())
        .map(str::to_owned)
        .ok_or_else(|| "action-input-invalid: output title is required".into())
}
