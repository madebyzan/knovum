use sqlx::SqlitePool;

use crate::spaces::db;

pub async fn update_session_options(
    pool: &SqlitePool,
    conversation_id: &str,
    options: serde_json::Value,
) -> Result<(), String> {
    let mut capabilities = load(pool, conversation_id).await?;
    capabilities
        .as_object_mut()
        .ok_or("external-session-capabilities-invalid")?
        .insert("configOptions".into(), options);
    save(pool, conversation_id, &capabilities).await
}

pub async fn update_session_mode(
    pool: &SqlitePool,
    conversation_id: &str,
    mode_id: &str,
) -> Result<(), String> {
    let mut capabilities = load(pool, conversation_id).await?;
    let modes = capabilities
        .get_mut("modes")
        .and_then(serde_json::Value::as_object_mut)
        .ok_or("external-session-modes-unavailable")?;
    modes.insert(
        "currentModeId".into(),
        serde_json::Value::String(mode_id.to_owned()),
    );
    save(pool, conversation_id, &capabilities).await
}

async fn load(pool: &SqlitePool, conversation_id: &str) -> Result<serde_json::Value, String> {
    let raw: String = sqlx::query_scalar(
        "SELECT session_capabilities FROM spaces_external_agent_conversations WHERE id=$1",
    )
    .bind(conversation_id)
    .fetch_one(pool)
    .await
    .map_err(db::sql_error)?;
    serde_json::from_str(&raw).map_err(db::json_error)
}

async fn save(
    pool: &SqlitePool,
    conversation_id: &str,
    value: &serde_json::Value,
) -> Result<(), String> {
    sqlx::query(
        "UPDATE spaces_external_agent_conversations SET session_capabilities=$1,
         updated_at=$2 WHERE id=$3 AND state='connected'",
    )
    .bind(serde_json::to_string(value).map_err(db::json_error)?)
    .bind(crate::clock::unix_time_millis()?)
    .bind(conversation_id)
    .execute(pool)
    .await
    .map_err(db::sql_error)?;
    Ok(())
}
