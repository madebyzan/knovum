use serde_json::Value;
use sqlx::{Sqlite, Transaction};

use crate::spaces::{db, speech::translation::WORKFLOW};

pub(super) async fn validate_completed_runs(
    tx: &mut Transaction<'_, Sqlite>,
    space_id: &str,
) -> Result<(), String> {
    let revisions: Vec<String> = sqlx::query_scalar(
        "SELECT r.output_config FROM spaces_speech_project_revisions r \
         JOIN spaces_speech_projects p ON p.id=r.project_id \
         WHERE p.space_id=? AND json_extract(r.output_config,'$.translation.runId') IS NOT NULL",
    )
    .bind(space_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for encoded in revisions {
        let config: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
        let run_id = config
            .pointer("/translation/runId")
            .and_then(Value::as_str)
            .ok_or("Video translation provenance Run is unavailable")?;
        let row: Option<(String, String)> = sqlx::query_as(
            "SELECT state,input_snapshot FROM spaces_runs WHERE id=? AND home_space_id=?",
        )
        .bind(run_id)
        .bind(space_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        let Some((state, input)) = row else {
            return Err("Video translation provenance references a missing Run".into());
        };
        let input: Value = serde_json::from_str(&input).map_err(db::json_error)?;
        if state != "succeeded" || input.get("workflow").and_then(Value::as_str) != Some(WORKFLOW) {
            return Err("Video translation provenance requires a succeeded translation Run".into());
        }
    }
    let runs: Vec<(String, String)> = sqlx::query_as(
        "SELECT id,input_snapshot FROM spaces_runs WHERE home_space_id=? AND state='succeeded' \
         AND json_extract(input_snapshot,'$.workflow')=?",
    )
    .bind(space_id)
    .bind(WORKFLOW)
    .fetch_all(&mut **tx)
    .await
    .map_err(db::sql_error)?;
    for (run_id, encoded) in runs {
        let input: Value = serde_json::from_str(&encoded).map_err(db::json_error)?;
        let expected = input
            .get("segments")
            .and_then(Value::as_array)
            .map(Vec::len)
            .ok_or("Video translation Run segments are unavailable")?;
        let checkpoints: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM spaces_run_steps WHERE run_id=? \
             AND node_kind='speech_video_translation_segment' AND state='succeeded' \
             AND json_extract(output,'$.stepId')=id",
        )
        .bind(&run_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(db::sql_error)?;
        if usize::try_from(checkpoints).ok() != Some(expected) {
            return Err("Video translation Run checkpoints are incomplete".into());
        }
    }
    Ok(())
}
