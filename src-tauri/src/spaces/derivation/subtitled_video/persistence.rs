use crate::app_error::AppError;
use serde_json::Value;
use sqlx::SqlitePool;
use tokio_util::sync::CancellationToken;

use super::super::video_output::RenderedVideo;
use super::super::{pending_output, relations, CompletedDerivationOutput};
use crate::spaces::{
    db,
    knowledge::{contracts, indexing::revision},
};

pub struct VideoSubtitleSource {
    pub path: String,
    pub srt: String,
}

pub async fn complete(
    pool: &SqlitePool,
    id: &str,
    output: RenderedVideo,
    authority: &super::super::action_execution::ExecutionAuthority,
    cancellation: &CancellationToken,
) -> Result<CompletedDerivationOutput, AppError> {
    let mut tx = db::begin_write(pool).await?;
    authority.require_active(&mut *tx).await?;
    if cancellation.is_cancelled() {
        return Err(super::CANCELLED.into());
    }
    let row: (String, String, String, String) = sqlx::query_as(
        "SELECT d.space_id,i.node_id,d.output_node_id,n.data FROM spaces_knowledge_derivations d \
         JOIN spaces_knowledge_derivation_inputs i ON i.derivation_id=d.id AND i.position=0 AND i.role='primary' \
         JOIN spaces_nodes n ON n.id=d.output_node_id WHERE d.id=$1 AND d.status='running'",
    ).bind(id).fetch_optional(&mut *tx).await.map_err(db::sql_error)?
        .ok_or("Subtitle derivation is no longer running")?;
    super::super::inputs::validate_current(&mut tx, id).await?;
    let now = crate::clock::unix_time_millis()?;
    let mut data: Value = serde_json::from_str(&row.3).map_err(db::json_error)?;
    pending_output::validate_unpublished_asset(&data)?;
    apply_output(&mut data, &output, now)?;
    contracts::validate_node(&mut tx, &row.0, "video", &data).await?;
    let relation = relations::derives_from(
        &row.0,
        &row.2,
        &row.1,
        "Subtitle video created from source video knowledge",
        now,
    );
    crate::spaces::derivation::persist_output_node(&mut tx, &row.2, &data, now).await?;
    relations::insert(&mut tx, &relation).await?;
    sqlx::query("UPDATE spaces_knowledge_derivations SET status='completed',completed_at=$1,updated_at=$1 WHERE id=$2")
        .bind(now).bind(id).execute(&mut *tx).await.map_err(db::sql_error)?;
    if cancellation.is_cancelled() {
        return Err(super::CANCELLED.into());
    }
    tx.commit().await.map_err(db::sql_error)?;
    Ok(CompletedDerivationOutput {
        node_id: row.2,
        data,
        relation,
    })
}

fn apply_output(data: &mut Value, output: &RenderedVideo, now: i64) -> Result<(), AppError> {
    let object = data
        .as_object_mut()
        .ok_or("Derived video data is invalid")?;
    object.insert("assetPath".into(), output.path.clone().into());
    object.insert("sourceRevision".into(), output.revision.clone().into());
    object.insert("size".into(), output.size.into());
    object.insert("modifiedAt".into(), now.into());
    object.remove("previewPath");
    object.remove("processingState");
    if let Some(knowledge) = object
        .get_mut("mediaKnowledge")
        .and_then(Value::as_object_mut)
    {
        knowledge.insert("sourceRevision".into(), output.revision.clone().into());
    }
    revision::apply(data).map(|_| ()).map_err(AppError::from)
}
