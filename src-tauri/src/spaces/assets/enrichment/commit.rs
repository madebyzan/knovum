use serde_json::Value;
use sqlx::Row;
use tauri::AppHandle;

use crate::spaces::{
    db,
    knowledge::indexing::{revision, KnowledgeChunkInput},
    nodes::{self, contracts::NodeContentChange},
};

pub(in crate::spaces) struct PublicationGuard<'a> {
    pub source_revision: Option<&'a str>,
    pub job_id: Option<&'a str>,
    pub media_knowledge: Option<&'a Value>,
}

/// Merge only fields owned by an enrichment into the current aggregate, in one write transaction.
/// Both source replacement and deletion invalidate an in-flight result before it can be published.
pub(in crate::spaces) async fn publish(
    app: &AppHandle,
    space_id: &str,
    node_id: &str,
    guard: PublicationGuard<'_>,
    fields: serde_json::Map<String, Value>,
    chunks: Option<Vec<KnowledgeChunkInput>>,
) -> Result<Value, String> {
    let PublicationGuard {
        source_revision,
        job_id,
        media_knowledge,
    } = guard;
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let row = sqlx::query(
        "SELECT data FROM spaces_nodes WHERE space_id=$1 AND id=$2 AND deleted_at IS NULL",
    )
    .bind(space_id)
    .bind(node_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?
    .ok_or("Enrichment target no longer exists")?;
    let raw: String = row.try_get("data").map_err(db::sql_error)?;
    let mut current: Value = serde_json::from_str(&raw).map_err(db::json_error)?;
    let expected_data = current.clone();
    if let Some(expected) = media_knowledge {
        super::media_knowledge::validate_input(&current["mediaKnowledge"], expected)?;
    }
    let current_revision = if job_id.is_some() && fields.contains_key("visualKnowledge") {
        super::jobs::revision::visual_revision(&current)?
    } else {
        current
            .get("sourceRevision")
            .and_then(Value::as_str)
            .map(str::to_owned)
    };
    if current_revision.as_deref() != source_revision {
        return Err("Enrichment source revision is obsolete".into());
    }
    if let Some(id) = job_id {
        let live: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spaces_enrichment_jobs WHERE id=$1 AND space_id=$2 AND node_id=$3 AND source_revision=$4 AND status='running'")
            .bind(id).bind(space_id).bind(node_id).bind(source_revision)
            .fetch_one(&mut *tx).await.map_err(db::sql_error)?;
        if live != 1 {
            return Err("Enrichment job is no longer running".into());
        }
    }
    let object = current
        .as_object_mut()
        .ok_or("Node data must be an object")?;
    for (key, value) in fields {
        if value.is_null() {
            object.remove(&key);
        } else {
            object.insert(key, value);
        }
    }
    revision::apply(&mut current)?;
    let now = crate::clock::unix_time_millis()?;
    nodes::update_many_in_transaction(
        &mut tx,
        space_id,
        &[NodeContentChange {
            id: node_id.into(),
            expected_data,
            data: current.clone(),
            source_chunks: chunks,
        }],
        "system",
        now,
    )
    .await?;
    sqlx::query("UPDATE spaces SET updated_at=$1 WHERE id=$2")
        .bind(now)
        .bind(space_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(current)
}
