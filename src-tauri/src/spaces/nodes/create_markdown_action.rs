use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};

use serde_json::{json, Value};
use sqlx::{Sqlite, Transaction};

use crate::{
    foundation::app_error::AppError,
    spaces::{
        assets::{markdown_media, to_webview_url},
        authorization::Principal,
        db,
        nodes::{self, contracts::InsertNode},
        structure::{self, SizingMode},
    },
};

const NODE_MEDIA_PREFIX: &str = "knovum-node://";
const MEDIA_LINEAGE: &str =
    "Embedded generated or existing Space media into this Markdown document";

pub(crate) async fn execute(
    tx: &mut Transaction<'_, Sqlite>,
    principal: &Principal,
    space_id: &str,
    parameters: &Value,
    now: i64,
) -> Result<Value, AppError> {
    let object = parameters
        .as_object()
        .ok_or("create-markdown parameters must be an object")?;
    let title = required_text(object, "title", 512)?;
    let raw_content = required_text(object, "content", 4 * 1024 * 1024)?;
    let body = match crate::spaces::assets::node_payload::extract_markdown_heading(raw_content) {
        Some((heading, rest)) if heading.trim() == title && !rest.trim().is_empty() => rest,
        _ => raw_content.to_owned(),
    };
    let (content, embedded_node_ids) = resolve_embedded_media(tx, space_id, &body).await?;

    let (mut snapshot, revision) = structure::repository::load_in_transaction(tx, space_id).await?;
    let (initial_x, initial_y) =
        structure::place_new_content(&snapshot.structure, &snapshot.layouts, 260.0, 184.0)?;
    let node_id = uuid::Uuid::new_v4().to_string();
    let mut node = InsertNode {
        id: node_id.clone(),
        kind: "markdown".into(),
        initial_x,
        initial_y,
        width: 260.0,
        height: 184.0,
        sizing_mode: SizingMode::Content,
        appearance: None,
        file_subpath: None,
        data: json!({
            "kind": "markdown",
            "title": title,
            "text": content,
        }),
        source_chunks: Vec::new(),
    };
    crate::spaces::knowledge::validation::normalize_node(tx, space_id, &node.kind, &mut node.data)
        .await?;
    let before = structure::history::capture(tx, space_id, &snapshot).await?;
    structure::creation::append_content(&mut snapshot, std::slice::from_ref(&node), None)?;
    nodes::insert_rows_with_actor(
        tx,
        space_id,
        std::slice::from_ref(&node),
        crate::spaces::knowledge::events::EventActor::new(
            principal.kind.event_actor_kind(),
            Some(&principal.id),
        ),
        now,
    )
    .await?;
    for source_id in &embedded_node_ids {
        crate::spaces::derivation::relations::derives_from(
            tx,
            space_id,
            &node_id,
            source_id,
            MEDIA_LINEAGE,
            now,
        )
        .await?;
    }
    let edit_revision =
        structure::repository::save_in_transaction(tx, space_id, revision, &mut snapshot).await?;
    let after = structure::history::capture(tx, space_id, &snapshot).await?;
    structure::history::record(tx, space_id, &before, &after).await?;
    Ok(json!({
        "nodeId": node_id,
        "editRevision": edit_revision,
        "principal": {"kind": principal.kind, "id": principal.id},
    }))
}

async fn resolve_embedded_media(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
    markdown: &str,
) -> Result<(String, Vec<String>), AppError> {
    let mut requested_ids = BTreeSet::new();
    let _ = markdown_media::rewrite_markdown_media(markdown, |dest_url| {
        if let Some(id) = dest_url.trim().strip_prefix(NODE_MEDIA_PREFIX) {
            let trimmed = id.trim();
            if !trimmed.is_empty() {
                requested_ids.insert(trimmed.to_owned());
            }
        }
        Ok(None)
    });
    if requested_ids.is_empty() {
        return Ok((markdown.to_owned(), Vec::new()));
    }

    let mut urls: BTreeMap<String, String> = BTreeMap::new();
    let mut embedded_ids = Vec::new();
    for media_node_id in requested_ids {
        let asset_path: Option<String> = sqlx::query_scalar(
            "SELECT json_extract(data,'$.assetPath') FROM spaces_nodes \
             WHERE space_id=$1 AND id=$2 AND kind IN ('image', 'video') AND deleted_at IS NULL",
        )
        .bind(space_id)
        .bind(&media_node_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(db::sql_error)?
        .flatten();
        let asset_path = asset_path.ok_or_else(|| {
            AppError::invalid(format!(
                "create-markdown referenced media node {media_node_id} was not found in this Space"
            ))
        })?;
        let url = to_webview_url(Path::new(&asset_path)).map_err(AppError::internal)?;
        urls.insert(media_node_id.clone(), url);
        embedded_ids.push(media_node_id);
    }

    let rewritten = markdown_media::rewrite_markdown_media(markdown, |dest_url| {
        Ok(dest_url
            .trim()
            .strip_prefix(NODE_MEDIA_PREFIX)
            .map(str::trim)
            .and_then(|id| urls.get(id).cloned()))
    })
    .map_err(AppError::internal)?;

    Ok((rewritten, embedded_ids))
}

fn required_text<'a>(
    parameters: &'a serde_json::Map<String, Value>,
    key: &str,
    max_utf16: usize,
) -> Result<&'a str, AppError> {
    let value = parameters
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("create-markdown requires {key}"))?;
    if value.encode_utf16().count() > max_utf16 {
        return Err(AppError::invalid(format!(
            "create-markdown {key} exceeds its limit"
        )));
    }
    Ok(value)
}
