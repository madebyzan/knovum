use serde_json::{json, Map, Value};
use tauri::AppHandle;

use super::inputs::ResolvedInputs;
use crate::foundation::app_error::AppError;
use crate::spaces::{
    db,
    knowledge::{indexing::KnowledgeChunkInput, model::revision},
    nodes::{self, contracts::InsertNode},
    structure::{self, SizingMode},
};

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
    if crate::spaces::actions::is_image_transform(action_id) {
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
    let Some((kind, mut data)) = build_pending_output_content(app, action_id, parameters, inputs)?
    else {
        return Ok(());
    };
    let node_id = format!("{}-output", authority.run_id);
    request.insert("outputNodeId".into(), node_id.clone().into());
    if node_exists(app, space_id, &node_id).await? {
        return Ok(());
    }
    revision::apply(&mut data)?;
    let sources = inputs
        .nodes
        .iter()
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    insert_output(app, space_id, node_id, kind, data, &sources, authority).await
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

async fn node_exists(app: &AppHandle, space_id: &str, node_id: &str) -> Result<bool, AppError> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
    )
    .bind(node_id)
    .bind(space_id)
    .fetch_one(db::pool(app).await?)
    .await
    .map_err(db::sql_error)?;
    Ok(count == 1)
}

async fn insert_output(
    app: &AppHandle,
    space_id: &str,
    node_id: String,
    kind: &str,
    payload: Value,
    sources: &[String],
    authority: &super::super::action_execution::ExecutionAuthority,
) -> Result<(), AppError> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    authority.require_active(&mut *tx).await?;
    let (mut snapshot, base) =
        structure::repository::load_in_transaction(&mut tx, space_id).await?;
    let before = structure::history::capture(&mut tx, space_id, &snapshot).await?;
    let (x, y) = structure::place_new_content(
        &snapshot.structure,
        &snapshot.layouts,
        sources,
        320.0,
        220.0,
    )?;
    let node = InsertNode {
        id: node_id,
        kind: kind.into(),
        initial_x: x,
        initial_y: y,
        width: 320.0,
        height: 220.0,
        sizing_mode: SizingMode::Content,
        appearance: None,
        file_subpath: None,
        data: payload,
        source_chunks: Vec::<KnowledgeChunkInput>::new(),
    };
    structure::creation::append_content(&mut snapshot, std::slice::from_ref(&node), None)?;
    nodes::insert_rows_with_actor(
        &mut tx,
        space_id,
        &[node],
        crate::spaces::knowledge::events::EventActor::new("system", Some(&authority.run_id)),
        crate::foundation::clock::unix_time_millis()?,
        crate::spaces::assets::enrichment::jobs::intent::selection(app)?.as_ref(),
    )
    .await?;
    let revision =
        structure::repository::save_in_transaction(&mut tx, space_id, base, &snapshot).await?;
    let after = structure::history::capture(&mut tx, space_id, &snapshot).await?;
    structure::history::record(&mut tx, space_id, &before, &after, None, None, revision).await?;
    Ok(tx.commit().await.map_err(db::sql_error)?)
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
