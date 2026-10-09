use tauri::{AppHandle, Runtime};

use super::contracts::DeleteSpaceRequest;
use crate::spaces::db;

pub async fn delete<R: Runtime>(
    app: &AppHandle<R>,
    request: DeleteSpaceRequest,
) -> Result<Vec<String>, String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let deletable = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM spaces WHERE id = $1 AND kind = 'space' AND archived_at IS NOT NULL",
    )
    .bind(&request.space_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if deletable == 0 {
        tx.rollback().await.map_err(db::sql_error)?;
        return Err("Space is not archived or no longer exists".to_owned());
    }

    let paths =
        crate::spaces::assets::referenced_paths_for_space(&mut tx, &request.space_id).await?;
    crate::spaces::knowledge::retrieval::invalidation_outbox::enqueue_space(
        &mut tx,
        &request.space_id,
        crate::clock::unix_time_millis()?,
    )
    .await?;
    let executed = sqlx::query(
        "DELETE FROM spaces WHERE id = $1 AND kind = 'space' AND archived_at IS NOT NULL",
    )
    .bind(&request.space_id)
    .execute(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    if executed.rows_affected() != 1 {
        return Err("Space changed while it was being permanently deleted".to_owned());
    }
    tx.commit().await.map_err(db::sql_error)?;
    let agent_cleanup =
        crate::spaces::agent_artifacts::workspace::space_directory(app, &request.space_id)
            .and_then(|workspace| crate::external_agents::artifacts::discard(&workspace))
            .map_err(|error| error.to_string());
    if let Err(error) = agent_cleanup {
        log::warn!("failed to clean up Agent workspace for deleted space: {error}");
    }
    crate::spaces::knowledge::retrieval::invalidation_outbox::drain_and_report(app, pool).await;
    let unreferenced = crate::spaces::assets::unreferenced_paths(pool, paths).await;
    unreferenced
}
