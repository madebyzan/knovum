use serde_json::Value;
use sqlx::{Row, Sqlite, Transaction};

use crate::spaces::db;

pub(super) struct Causality {
    pub root_id: String,
    pub depth: u8,
    pub trace: Vec<String>,
}

pub(super) async fn load(
    tx: &mut Transaction<'_, Sqlite>,
    parent_run_id: Option<&str>,
) -> Result<Causality, String> {
    let Some(run_id) = parent_run_id else {
        return Ok(root());
    };
    let mut current_run_id = run_id.to_owned();
    for _ in 0..8 {
        let row = sqlx::query("SELECT input_snapshot,parent_run_id FROM spaces_runs WHERE id=$1")
            .bind(&current_run_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(db::sql_error)?;
        let Some(row) = row else { return Ok(root()) };
        let input: Value = serde_json::from_str(
            &row.try_get::<String, _>("input_snapshot")
                .map_err(db::sql_error)?,
        )
        .map_err(db::json_error)?;
        if let Some(event) = input.get("automationEvent") {
            return Ok(Causality {
                root_id: event
                    .get("causalRootId")
                    .and_then(Value::as_str)
                    .unwrap_or(&current_run_id)
                    .to_owned(),
                depth: event
                    .get("causalDepth")
                    .and_then(Value::as_u64)
                    .and_then(|value| u8::try_from(value).ok())
                    .unwrap_or(0),
                trace: event
                    .get("triggerTrace")
                    .cloned()
                    .map(serde_json::from_value)
                    .transpose()
                    .map_err(db::json_error)?
                    .unwrap_or_default(),
            });
        }
        let parent: Option<String> = row.try_get("parent_run_id").map_err(db::sql_error)?;
        let Some(parent) = parent else {
            return Ok(root());
        };
        current_run_id = parent;
    }
    Ok(root())
}

fn root() -> Causality {
    Causality {
        root_id: uuid::Uuid::new_v4().to_string(),
        depth: 0,
        trace: Vec::new(),
    }
}
