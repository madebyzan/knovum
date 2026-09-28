use serde_json::{json, Map, Value};
use tauri::AppHandle;

use super::inputs::ResolvedInputs;
use crate::spaces::{
    db,
    knowledge::indexing::{revision, KnowledgeChunkInput},
    nodes::{self, contracts::InsertNode},
    structure::{self, SizingMode},
};

mod media;

pub(super) async fn reserve(
    app: &AppHandle,
    space_id: &str,
    run_id: &str,
    action_id: &str,
    parameters: &Value,
    inputs: &ResolvedInputs,
    request: &mut Map<String, Value>,
) -> Result<(), String> {
    match action_id {
        "extract-subtitles" => media::subtitle_request(inputs, request)?,
        "render-subtitled-video" => media::subtitled_video_request(inputs, request)?,
        "render-presentation" => media::presentation_request(inputs, request)?,
        _ => {}
    }
    let Some((kind, mut data)) = build_pending_output_content(app, action_id, parameters, inputs)?
    else {
        return Ok(());
    };
    let node_id = format!("{run_id}-output");
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
    insert_output(app, space_id, run_id, node_id, kind, data, &sources).await
}

fn build_pending_output_content(
    app: &AppHandle,
    action_id: &str,
    parameters: &Value,
    inputs: &ResolvedInputs,
) -> Result<Option<(&'static str, Value)>, String> {
    let title = title(action_id, parameters);
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
        "render-subtitled-video" => (
            "video",
            media::build_subtitled_video_content(inputs, &title)?,
        ),
        "render-presentation" => (
            "file",
            json!({
                "kind":"file","title":title,"fileName":format!("{title}.pptx"),
                "assetPath": super::super::output_paths::reserve(app, "presentation", "pptx")?,
                "processingState":"processing"
            }),
        ),
        "extract-subtitles" => ("subtitle", media::build_subtitle_content(inputs)?),
        _ => return Ok(None),
    };
    Ok(Some(output))
}

async fn node_exists(app: &AppHandle, space_id: &str, node_id: &str) -> Result<bool, String> {
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
    run_id: &str,
    node_id: String,
    kind: &str,
    data: Value,
    sources: &[String],
) -> Result<(), String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
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
        data,
        source_chunks: Vec::<KnowledgeChunkInput>::new(),
    };
    structure::creation::append_content(&mut snapshot, std::slice::from_ref(&node), None)?;
    nodes::insert_rows_with_actor(
        &mut tx,
        space_id,
        &[node],
        "system",
        Some(run_id),
        crate::clock::unix_time_millis()?,
    )
    .await?;
    let revision =
        structure::repository::save_in_transaction(&mut tx, space_id, base, &snapshot).await?;
    let after = structure::history::capture(&mut tx, space_id, &snapshot).await?;
    structure::history::record(&mut tx, space_id, &before, &after, None, None, revision).await?;
    tx.commit().await.map_err(db::sql_error)
}

fn title(action_id: &str, parameters: &Value) -> String {
    parameters
        .get("title")
        .and_then(Value::as_str)
        .map(str::to_owned)
        .unwrap_or_else(|| {
            action_id
                .split('-')
                .map(capitalize)
                .collect::<Vec<_>>()
                .join(" ")
        })
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + chars.as_str())
        .unwrap_or_default()
}
