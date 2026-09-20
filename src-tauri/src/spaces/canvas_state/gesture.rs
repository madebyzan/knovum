use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Runtime};

use super::SpaceEditorDocument;
use crate::spaces::{db, nodes};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CommitCanvasGestureRequest {
    pub(super) space_id: String,
    pub(super) changes: Vec<nodes::contracts::NodeDataChange>,
    pub(super) base_revision: i64,
    pub(super) document: SpaceEditorDocument,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CommitCanvasGestureResult {
    revision: i64,
}

pub(crate) async fn commit<R: Runtime>(
    app: AppHandle<R>,
    request: CommitCanvasGestureRequest,
) -> Result<CommitCanvasGestureResult, String> {
    let pool = db::pool(&app).await?;
    commit_in_transaction(pool, &request).await
}

pub(super) async fn commit_in_transaction(
    pool: &sqlx::SqlitePool,
    request: &CommitCanvasGestureRequest,
) -> Result<CommitCanvasGestureResult, String> {
    let mut tx = db::begin_write(pool).await?;
    let now = crate::clock::unix_time_millis()?;
    nodes::update_many_in_transaction(&mut tx, &request.space_id, &request.changes, "human", now)
        .await?;
    let revision = super::save_in_transaction(
        &mut tx,
        &request.space_id,
        request.base_revision,
        &request.document,
    )
    .await?;
    nodes::touch_updated_at(&mut tx, &request.space_id, now).await?;
    tx.commit().await.map_err(db::sql_error)?;
    Ok(CommitCanvasGestureResult { revision })
}
