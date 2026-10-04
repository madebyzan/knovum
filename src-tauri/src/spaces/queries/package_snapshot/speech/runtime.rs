use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use crate::spaces::db;

pub(super) async fn validate_completed_runs(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<(), String> {
    let snapshots: Vec<String> = sqlx::query_scalar(
        "SELECT input_snapshot FROM spaces_runs WHERE home_space_id=? AND completed_at IS NOT NULL \
         AND json_extract(input_snapshot,'$.workflow')='speech_project_generation_v1'",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for encoded in snapshots {
        let snapshot: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
        let digest = snapshot
            .get("runtimeDigest")
            .and_then(Value::as_str)
            .ok_or("Speech Run has no frozen Python runtime digest")?;
        if digest.len() != 64
            || !digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err("Speech Run Python runtime digest is invalid".into());
        }
    }
    Ok(())
}
