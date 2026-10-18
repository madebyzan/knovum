use serde_json::Value;
use sqlx::SqlitePool;
use tokio_util::sync::CancellationToken;

use super::super::inheritance::DerivedOutput;
use super::super::video_output::{self, RenderedVideo};
use super::super::{pending_output, CompletedDerivationOutput};
use crate::foundation::app_error::AppError;
use crate::spaces::{db, knowledge::model::revision};

pub struct VideoSubtitleSource {
    pub path: String,
    pub srt: String,
}

pub async fn complete(
    pool: &SqlitePool,
    derivation_id: &str,
    output: RenderedVideo,
    authority: &super::super::action_execution::ExecutionAuthority,
    cancellation: &CancellationToken,
    enrichment_target: Option<&crate::ai::tasks::TaskTargetSelection>,
) -> Result<CompletedDerivationOutput, AppError> {
    let mut tx = db::begin_write(pool).await?;
    authority.require_active(&mut *tx).await?;
    if cancellation.is_cancelled() {
        return Err(super::CANCELLED.into());
    }
    let row: (String, String, String, String, String) = sqlx::query_as(
        "SELECT d.space_id,i.node_id,d.output_node_id,n.data,source.data FROM spaces_knowledge_derivations d \
         JOIN spaces_knowledge_derivation_inputs i ON i.derivation_id=d.id AND i.position=0 AND i.role='primary' \
         JOIN spaces_nodes source ON source.id=i.node_id AND source.space_id=d.space_id \
         JOIN spaces_nodes n ON n.id=d.output_node_id WHERE d.id=$1 AND d.status='running'",
    ).bind(derivation_id).fetch_optional(&mut *tx).await.map_err(db::sql_error)?
        .ok_or("Subtitle derivation is no longer running")?;
    super::super::inputs::validate_current(&mut tx, derivation_id).await?;
    let now = crate::foundation::clock::unix_time_millis()?;
    let mut data: Value = serde_json::from_str(&row.3).map_err(db::json_error)?;
    let source: Value = serde_json::from_str(&row.4).map_err(db::json_error)?;
    pending_output::validate_unpublished_asset(&data)?;
    apply_output(&mut data, &source, &output, now)?;
    crate::spaces::knowledge::validation::validate_node(&mut tx, &row.0, "video", &data).await?;
    crate::spaces::derivation::persist_output_node(&mut tx, &row.2, &data, now, enrichment_target)
        .await?;
    sqlx::query("UPDATE spaces_knowledge_derivations SET status='completed',completed_at=$1,updated_at=$1 WHERE id=$2")
        .bind(now).bind(derivation_id).execute(&mut *tx).await.map_err(db::sql_error)?;
    if cancellation.is_cancelled() {
        return Err(super::CANCELLED.into());
    }
    tx.commit().await.map_err(db::sql_error)?;
    Ok(CompletedDerivationOutput {
        node_id: row.2,
        data,
    })
}

fn apply_output(
    payload: &mut Value,
    source: &Value,
    output: &RenderedVideo,
    now: i64,
) -> Result<(), AppError> {
    let object = payload
        .as_object_mut()
        .ok_or("Derived video data is invalid")?;
    video_output::apply_published(object, output, now);
    DerivedOutput::SubtitledVideo.inherit(payload, source)?;
    revision::apply(payload).map(|_| ()).map_err(AppError::from)
}
