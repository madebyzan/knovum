use serde_json::Value;
use sqlx::{Sqlite, Transaction};
use tauri::AppHandle;

use crate::app_error::AppError;
use crate::spaces::{authorization::Principal, db};

pub(crate) async fn execute_immediate(
    tx: &mut Transaction<'_, Sqlite>,
    principal: &Principal,
    invocation_id: &str,
    space_id: &str,
    action_id: &str,
    parameters: &Value,
    now: i64,
) -> Result<Value, AppError> {
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
            Ok(serde_json::to_value(receipt).map_err(db::json_error)?)
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
            Ok(serde_json::to_value(page).map_err(db::json_error)?)
        }
        "search-knowledge" => {
            let query = parameters["query"]
                .as_str()
                .ok_or("search-knowledge requires query")?;
            let limit = parameters["limit"]
                .as_i64()
                .ok_or("search-knowledge requires limit")?;
            Ok(
                crate::spaces::queries::knowledge_projection::search_knowledge(
                    tx, space_id, query, limit,
                )
                .await?,
            )
        }
        "read-node" => {
            let node_id = parameters["nodeId"]
                .as_str()
                .ok_or("read-node requires nodeId")?;
            Ok(
                crate::spaces::queries::knowledge_projection::get_node(tx, space_id, node_id)
                    .await?,
            )
        }
        _ => Err("action-executor-unavailable: immediate Action has no registered handler".into()),
    }
}

pub(crate) async fn execute_durable(
    app: &AppHandle,
    action_id: &str,
    claim: &crate::spaces::runs::AttemptClaim,
    parameters: &Value,
    inputs: &Value,
) -> Result<Value, AppError> {
    crate::spaces::derivation::execute_action(
        app,
        action_id,
        &claim.home_space_id,
        parameters,
        inputs,
        crate::spaces::derivation::ExecutionAuthority {
            run_id: claim.run_id.clone(),
            execution_epoch: claim.execution_epoch,
        },
        &claim.cancellation,
    )
    .await
}
