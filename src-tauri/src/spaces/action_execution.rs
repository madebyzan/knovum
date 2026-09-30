use serde_json::Value;
use sqlx::{Sqlite, Transaction};
use tauri::AppHandle;

use crate::spaces::{authorization::Principal, db};

pub(crate) async fn execute_immediate(
    tx: &mut Transaction<'_, Sqlite>,
    principal: &Principal,
    invocation_id: &str,
    space_id: &str,
    action_id: &str,
    parameters: &Value,
    now: i64,
) -> Result<Value, String> {
    match action_id {
        "create-note" => {
            crate::spaces::nodes::create_note_action::execute(
                tx, principal, space_id, parameters, now,
            )
            .await
        }
        "commit-knowledge-changes" => {
            crate::spaces::knowledge::action::commit_changes(
                tx, principal, space_id, parameters, now,
            )
            .await
        }
        "edit-space-structure" => {
            let expected_edit_revision = parameters
                .get("expectedEditRevision")
                .and_then(Value::as_i64)
                .ok_or("space-edit-input-invalid: expectedEditRevision must be an integer")?;
            let command = serde_json::from_value(
                parameters
                    .get("command")
                    .cloned()
                    .ok_or("space-edit-input-invalid: command is required")?,
            )
            .map_err(db::json_error)?;
            let receipt = crate::spaces::structure::commands::execute_in_transaction(
                tx,
                crate::spaces::structure::ExecuteSpaceEditRequest {
                    space_id: space_id.to_owned(),
                    mutation_id: invocation_id.to_owned(),
                    expected_edit_revision,
                    command,
                },
            )
            .await?;
            serde_json::to_value(receipt).map_err(db::json_error)
        }
        "list-space-contents" => {
            let offset = parameters["offset"]
                .as_i64()
                .ok_or("list-space-contents requires offset")?;
            let limit = parameters["limit"]
                .as_i64()
                .ok_or("list-space-contents requires limit")?;
            let page = crate::spaces::queries::knowledge_projection::list_contents(
                tx, space_id, offset, limit,
            )
            .await?;
            serde_json::to_value(page).map_err(db::json_error)
        }
        "search-knowledge" => {
            crate::spaces::knowledge::action::search(tx, space_id, parameters).await
        }
        "read-node" => crate::spaces::knowledge::action::read_node(tx, space_id, parameters).await,
        _ => Err("action-executor-unavailable: immediate Action has no registered handler".into()),
    }
}

pub(crate) async fn execute_durable(
    app: &AppHandle,
    action_id: &str,
    run_id: &str,
    execution_epoch: i64,
    space_id: &str,
    parameters: &Value,
    inputs: &Value,
) -> Result<Value, String> {
    crate::spaces::derivation::execute_action(
        app,
        action_id,
        run_id,
        execution_epoch,
        space_id,
        parameters,
        inputs,
    )
    .await
}
