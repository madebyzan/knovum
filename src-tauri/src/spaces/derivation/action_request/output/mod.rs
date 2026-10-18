use serde_json::{json, Map, Value};
use tauri::AppHandle;

use super::inputs::ResolvedInputs;
use crate::foundation::app_error::AppError;
use crate::spaces::actions::SpaceActionInputRole;

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
        _ => {}
    }
    let Some(output) = build_pending_output_content(app, action_id, parameters, inputs)? else {
        return Ok(());
    };
    let node_id = format!("{}-output", authority.run_id);
    request.insert("outputNodeId".into(), node_id.clone().into());
    if super::super::pending_output::exists(crate::spaces::db::pool(app).await?, space_id, &node_id)
        .await?
    {
        return Ok(());
    }
    super::super::pending_output::reserve(
        app,
        space_id,
        vec![super::super::pending_output::PendingOutput {
            node_id,
            kind: output.kind,
            data: output.data,
        }],
        &inputs
            .nodes
            .iter()
            .map(|node| node.id.clone())
            .collect::<Vec<_>>(),
        output.lineage,
        authority,
    )
    .await
}

/// The content an Action reserves for its output, and why that output derives from its inputs.
/// One place decides both: the lineage is recorded with the node, so it cannot describe something
/// the output is not.
struct PendingOutputContent {
    kind: &'static str,
    data: Value,
    lineage: &'static str,
}

fn build_pending_output_content(
    app: &AppHandle,
    action_id: &str,
    parameters: &Value,
    inputs: &ResolvedInputs,
) -> Result<Option<PendingOutputContent>, AppError> {
    let title = title(parameters, inputs)?;
    if crate::spaces::actions::is_image_transform(action_id) {
        return Ok(Some(PendingOutputContent {
            kind: "image",
            data: generated_image_content(app, &title)?,
            lineage: image_transform_lineage(action_id)?,
        }));
    }
    let output = match action_id {
        "draft-document" | "draft-script" | "summarize-meeting" | "draft-social-post" => {
            PendingOutputContent {
                kind: "markdown",
                data: json!({
                    "kind":"markdown","title":title,"text":"","derivationState":"pending"
                }),
                lineage: "Grounded document drafted from selected knowledge",
            }
        }
        "generate-visualization" | "generate-image" => PendingOutputContent {
            kind: "image",
            data: generated_image_content(app, &title)?,
            lineage: "Visualization generated from selected knowledge",
        },
        "split-image-layers" => PendingOutputContent {
            kind: "image",
            data: generated_image_content(app, &title)?,
            lineage: super::super::image_layers::LINEAGE,
        },
        "generate-video" => PendingOutputContent {
            kind: "video",
            data: json!({
                "kind":"video","title":title,"fileName":format!("{title}.mp4"),
                "assetPath": super::super::output_paths::reserve(app, "generated", "mp4")?,
                "processingState":"processing"
            }),
            lineage: "Video generated from selected knowledge",
        },
        "add-video-subtitles" => PendingOutputContent {
            kind: "video",
            data: media::build_subtitled_video_content(app, &title)?,
            lineage: "Subtitle video created from source video knowledge",
        },
        _ => return Ok(None),
    };
    Ok(Some(output))
}

/// Why a transformed picture derives from its source, in the words of the transform that was asked
/// for. The transform is chosen by the Action, so the Action names the lineage; an image transform
/// the engine cannot explain is a gap rather than a default.
fn image_transform_lineage(action_id: &str) -> Result<&'static str, AppError> {
    match action_id {
        "upscale-image" => Ok("Image upscaled from source image"),
        "generate-image-angle" => Ok("View generated from source image"),
        "remove-image-object" => Ok("Object removed and background completed from source image"),
        "remove-image-background" => Ok("Background removed from source image"),
        "extract-image-selection" => Ok("Selection extracted from source image"),
        "generate-image-depth" => Ok("Relative inverse depth estimated from source image"),
        _ => Err(format!("Image transform {action_id} states no lineage").into()),
    }
}

fn generated_image_content(app: &AppHandle, title: &str) -> Result<Value, AppError> {
    Ok(json!({
        "kind":"image","title":title,"fileName":format!("{title}.png"),
        "assetPath": super::super::output_paths::reserve(app, "generated", "png")?,
        "processingState":"processing"
    }))
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
