mod contract;
mod events;
mod repository;
mod schedule;
mod tick;

pub use contract::{AutomationOccurrence, AutomationTrigger, SaveAutomationTriggerRequest};
pub(crate) use events::record_action_event;
pub(crate) use tick::run_due;

#[tauri::command]
pub async fn save_automation_trigger(
    app: tauri::AppHandle,
    request: SaveAutomationTriggerRequest,
) -> Result<AutomationTrigger, String> {
    repository::save(crate::spaces::db::pool(&app).await?, request).await
}

#[tauri::command]
pub async fn list_automation_triggers(
    app: tauri::AppHandle,
    space_id: String,
) -> Result<Vec<AutomationTrigger>, String> {
    repository::list(crate::spaces::db::pool(&app).await?, &space_id).await
}

#[tauri::command]
pub async fn set_automation_trigger_enabled(
    app: tauri::AppHandle,
    trigger_id: String,
    enabled: bool,
) -> Result<(), String> {
    repository::set_enabled(crate::spaces::db::pool(&app).await?, &trigger_id, enabled).await
}

#[tauri::command]
pub async fn renew_automation_trigger(
    app: tauri::AppHandle,
    trigger_id: String,
    valid_until: i64,
) -> Result<(), String> {
    repository::renew(
        crate::spaces::db::pool(&app).await?,
        &trigger_id,
        valid_until,
    )
    .await
}

#[tauri::command]
pub async fn delete_automation_trigger(
    app: tauri::AppHandle,
    trigger_id: String,
) -> Result<(), String> {
    repository::delete(crate::spaces::db::pool(&app).await?, &trigger_id).await
}

#[tauri::command]
pub async fn test_automation_trigger(
    app: tauri::AppHandle,
    trigger_id: String,
) -> Result<String, String> {
    tick::test_trigger(crate::spaces::db::pool(&app).await?, &trigger_id).await
}

#[tauri::command]
pub async fn list_automation_occurrences(
    app: tauri::AppHandle,
    space_id: String,
) -> Result<Vec<AutomationOccurrence>, String> {
    repository::list_occurrences(crate::spaces::db::pool(&app).await?, &space_id).await
}

#[tauri::command]
pub fn preview_automation_schedule(
    schedule: String,
    timezone: String,
    count: u8,
) -> Result<Vec<i64>, String> {
    self::schedule::preview(&schedule, &timezone, count)
}
