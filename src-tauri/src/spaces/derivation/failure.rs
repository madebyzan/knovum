use serde_json::Value;
use tauri::AppHandle;

use crate::spaces::db;

pub enum PendingNodeState {
    Processing,
    Derivation,
}

impl PendingNodeState {
    pub const fn field_name(&self) -> &'static str {
        match self {
            Self::Processing => "processingState",
            Self::Derivation => "derivationState",
        }
    }
}

pub fn state_field_for(action_kind: &str) -> &'static str {
    if action_kind == "draft-document" {
        "derivationState"
    } else {
        "processingState"
    }
}

pub(super) async fn fail_message(app: &AppHandle, id: &str, message: &str) -> Result<(), String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    mark_failed(&mut tx, id, message).await?;
    tx.commit().await.map_err(db::sql_error)
}

pub async fn fail_pending(
    app: &AppHandle,
    space_id: &str,
    node_id: &str,
    field: PendingNodeState,
) -> Result<(), String> {
    let pool = db::pool(app).await?;
    let mut tx = db::begin_write(pool).await?;
    let encoded = sqlx::query_scalar::<_, String>(
        "SELECT data FROM spaces_nodes WHERE id=$1 AND space_id=$2 AND deleted_at IS NULL",
    )
    .bind(node_id)
    .bind(space_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(db::sql_error)?;
    let Some(encoded) = encoded else {
        return Ok(());
    };
    let mut data: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
    let key = field.field_name();
    data[key] = "failed".into();
    sqlx::query("UPDATE spaces_nodes SET data=$1,updated_at=$2 WHERE id=$3 AND space_id=$4")
        .bind(serde_json::to_string(&data).map_err(db::json_error)?)
        .bind(crate::clock::unix_time_millis()?)
        .bind(node_id)
        .bind(space_id)
        .execute(&mut *tx)
        .await
        .map_err(db::sql_error)?;
    tx.commit().await.map_err(db::sql_error)
}

pub(crate) async fn reconcile_interrupted(app: &AppHandle) -> Result<(), String> {
    let pool = db::pool(app).await?;
    let rows: Vec<(String, String)> = sqlx::query_as(
        "SELECT id,action_kind FROM spaces_knowledge_derivations WHERE status='running'",
    )
    .fetch_all(pool)
    .await
    .map_err(db::sql_error)?;
    for (id, action) in rows {
        let mut tx = db::begin_write(pool).await?;
        mark_failed(&mut tx, &id, &format!("{action} was interrupted")).await?;
        tx.commit().await.map_err(db::sql_error)?;
    }
    Ok(())
}

async fn mark_failed(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    id: &str,
    error: &str,
) -> Result<(), String> {
    let now = crate::clock::unix_time_millis()?;
    let row: Option<(String, String)> = sqlx::query_as(
        "SELECT n.data,d.action_kind FROM spaces_knowledge_derivations d JOIN spaces_nodes n ON n.id=d.output_node_id WHERE d.id=$1 AND d.status='running'",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    let Some((data, action_kind)) = row else {
        return Ok(());
    };
    let mut value: Value = serde_json::from_str(&data).map_err(db::json_error)?;
    let state_field = state_field_for(&action_kind);
    value[state_field] = Value::String("failed".into());
    let encoded = serde_json::to_string(&value).map_err(db::json_error)?;
    sqlx::query("UPDATE spaces_nodes SET data=$1,updated_at=$2 WHERE id=(SELECT output_node_id FROM spaces_knowledge_derivations WHERE id=$3)")
        .bind(encoded).bind(now).bind(id).execute(&mut **tx).await.map_err(db::sql_error)?;
    sqlx::query("UPDATE spaces_knowledge_derivations SET status='failed',error=$1,completed_at=$2,updated_at=$2 WHERE id=$3 AND status='running'")
        .bind(error).bind(now).bind(id).execute(&mut **tx).await.map_err(db::sql_error)?;
    Ok(())
}
