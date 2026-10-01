use crate::app_error::AppError;
use serde_json::{json, Value};
use sqlx::{Sqlite, Transaction};

use crate::spaces::{
    authorization::Principal,
    knowledge::contracts,
    nodes::{self, contracts::InsertNode},
    structure::{self, SizingMode},
};

pub(crate) async fn execute(
    tx: &mut Transaction<'_, Sqlite>,
    principal: &Principal,
    space_id: &str,
    parameters: &Value,
    now: i64,
) -> Result<Value, AppError> {
    let object = parameters
        .as_object()
        .ok_or("create-note parameters must be an object")?;
    let title = required_text(object, "title", 512)?;
    let content = required_text(object, "content", 4 * 1024 * 1024)?;
    let (mut snapshot, revision) = structure::repository::load_in_transaction(tx, space_id).await?;
    let (initial_x, initial_y) =
        structure::place_new_content(&snapshot.structure, &snapshot.layouts, &[], 320.0, 220.0)?;
    let node_id = uuid::Uuid::new_v4().to_string();
    let mut node = InsertNode {
        id: node_id.clone(),
        kind: "note".into(),
        initial_x,
        initial_y,
        width: 320.0,
        height: 220.0,
        sizing_mode: SizingMode::Content,
        appearance: None,
        file_subpath: None,
        data: json!({
            "kind": "note", "title": title, "plainText": content,
            "tiptapContent": tiptap_document(content),
        }),
        source_chunks: Vec::new(),
    };
    contracts::normalize_node(tx, space_id, &node.kind, &mut node.data).await?;
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
        None, // Notes do not create visual enrichment.
    )
    .await?;
    let edit_revision =
        structure::repository::save_in_transaction(tx, space_id, revision, &snapshot).await?;
    let after = structure::history::capture(tx, space_id, &snapshot).await?;
    structure::history::record(tx, space_id, &before, &after, None, None, edit_revision).await?;
    Ok(json!({
        "nodeId": node_id, "editRevision": edit_revision,
        "principal": {"kind": principal.kind, "id": principal.id},
    }))
}

fn tiptap_document(content: &str) -> Value {
    let paragraphs = content
        .lines()
        .map(|line| {
            if line.is_empty() {
                json!({"type": "paragraph"})
            } else {
                json!({"type": "paragraph", "content": [{"type": "text", "text": line}]})
            }
        })
        .collect::<Vec<_>>();
    json!({"type": "doc", "content": paragraphs})
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
        .ok_or_else(|| format!("create-note requires {key}"))?;
    if value.encode_utf16().count() > max_utf16 {
        return Err(AppError::invalid(format!(
            "create-note {key} exceeds its limit"
        )));
    }
    Ok(value)
}
