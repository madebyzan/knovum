use serde_json::{json, Value};
use sqlx::{Row, SqliteConnection};

use super::chunks::{query_slice, ChunkReadFilter};
use crate::spaces::db;

const PREVIEW_CHUNK_LIMIT: i64 = 20;
const UI_ONLY_FIELDS: &[&str] = &[
    "previewPath",
    "faviconPath",
    "previewImagePath",
    "waveform",
    "waveformPeaks",
];

pub(crate) async fn load_node(
    connection: &mut SqliteConnection,
    space_id: &str,
    node_id: &str,
) -> Result<Value, String> {
    let row = sqlx::query(
        "SELECT id,kind,data,created_at,updated_at FROM spaces_nodes \
         WHERE space_id=$1 AND id=$2 AND deleted_at IS NULL AND kind NOT IN ('credential','portal')",
    )
    .bind(space_id)
    .bind(node_id)
    .fetch_optional(&mut *connection)
    .await
    .map_err(db::sql_error)?
    .ok_or("node-unavailable: node does not exist in the requested Space")?;
    let raw_payload = parse_json(row.try_get("data").map_err(db::sql_error)?)?;
    let asset_revision = asset_revision(&raw_payload)?;
    let knowledge_revision = raw_payload.get("knowledgeRevision").cloned();
    let source_revision = raw_payload.get("sourceRevision").cloned();
    let projected_payload = project_agent_payload(raw_payload);

    let slice = query_slice(
        connection,
        space_id,
        node_id,
        &ChunkReadFilter {
            role: None,
            offset: 0,
            limit: PREVIEW_CHUNK_LIMIT,
            around_chunk_id: None,
            context_chunks: None,
        },
    )
    .await?;

    let mut response = json!({
        "id": row.try_get::<String, _>("id").map_err(db::sql_error)?,
        "kind": row.try_get::<String, _>("kind").map_err(db::sql_error)?,
        "knowledgeRevision": knowledge_revision,
        "sourceRevision": source_revision,
        "assetRevision": asset_revision,
        "data": projected_payload,
        "totalChunks": slice.total_chunks,
        "chunks": slice.chunks,
        "createdAt": row.try_get::<i64, _>("created_at").map_err(db::sql_error)?,
        "updatedAt": row.try_get::<i64, _>("updated_at").map_err(db::sql_error)?,
    });
    if let Some(next_offset) = slice.next_offset {
        response["nextChunksOffset"] = Value::from(next_offset);
    }
    if !slice.roles.is_empty() {
        response["chunkRoles"] = Value::Object(slice.roles);
    }
    Ok(response)
}

/// Strips redundant editor ASTs, word-level transcript arrays, and UI-only preview fields
/// so Agents read high-signal semantic content without wasting context tokens.
fn project_agent_payload(mut payload: Value) -> Value {
    let Some(object) = payload.as_object_mut() else {
        return payload;
    };
    for key in UI_ONLY_FIELDS {
        object.remove(*key);
    }
    if object.get("plainText").and_then(Value::as_str).is_some() {
        object.remove("tiptapContent");
    }
    if let Some(media) = object
        .get_mut("mediaKnowledge")
        .and_then(Value::as_object_mut)
    {
        if let Some(transcript) = media.remove("transcript") {
            let segment_count = transcript
                .get("segments")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            let speaker_count = transcript
                .get("speakerTracks")
                .and_then(Value::as_array)
                .map_or(0, Vec::len);
            media.insert(
                "transcriptMetadata".into(),
                json!({
                    "id": transcript.get("id"),
                    "revision": transcript.get("revision"),
                    "durationMs": transcript.get("durationMs"),
                    "segmentCount": segment_count,
                    "speakerCount": speaker_count,
                }),
            );
        }
    }
    payload
}

fn asset_revision(payload: &Value) -> Result<Option<String>, String> {
    if payload.get("assetPath").is_none() || payload.get("sourceRevision").is_none() {
        return Ok(None);
    }
    let kind = payload["kind"]
        .as_str()
        .ok_or("Resource kind is unavailable")?;
    use crate::spaces::derivation::input_revision::{resolve, RevisionBasis};
    let basis = if kind == "image" {
        RevisionBasis::Rendered
    } else {
        RevisionBasis::Asset
    };
    resolve(kind, payload, basis).map(|(revision, _)| Some(revision))
}

fn parse_json(raw: String) -> Result<Value, String> {
    serde_json::from_str(&raw).map_err(db::json_error)
}
