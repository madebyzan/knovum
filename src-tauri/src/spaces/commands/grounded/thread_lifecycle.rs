use tauri::{AppHandle, Runtime};

use crate::spaces::{db, knowledge::grounding};

#[tauri::command]
pub async fn update_grounded_thread_title<R: Runtime>(
    app: AppHandle<R>,
    thread_id: String,
    title: String,
) -> Result<(), String> {
    grounding::validate_title(&title)?;
    let now = crate::clock::unix_time_millis()?;
    let pool = db::pool(&app).await?;
    let executed =
        sqlx::query("UPDATE spaces_ai_threads SET title = ?, updated_at = ? WHERE id = ?")
            .bind(title)
            .bind(now)
            .bind(thread_id)
            .execute(pool)
            .await
            .map_err(db::sql_error)?;
    db::require_one(executed, "Grounded AI thread no longer exists")
}

#[tauri::command]
pub async fn delete_grounded_thread<R: Runtime>(
    app: AppHandle<R>,
    thread_id: String,
) -> Result<(), String> {
    let pool = db::pool(&app).await?;
    let executed = sqlx::query("DELETE FROM spaces_ai_threads WHERE id = ?")
        .bind(thread_id)
        .execute(pool)
        .await
        .map_err(db::sql_error)?;
    db::require_one(executed, "Grounded AI thread no longer exists")
}
