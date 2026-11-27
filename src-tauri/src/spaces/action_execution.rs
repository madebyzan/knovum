use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::{Sqlite, Transaction};
use tauri::AppHandle;

use crate::foundation::app_error::AppError;
use crate::spaces::{
    actions::catalog::parameters::{
        ContentsParameters, NodeChunksParameters, NodeParameters, RelationsParameters,
        SearchParameters,
    },
    authorization::Principal,
    db,
    queries::knowledge_projection::{self, ChunkReadFilter, SearchFilter},
};

pub(crate) struct ImmediateActionRequest<'a> {
    pub(crate) principal: &'a Principal,
    pub(crate) run_id: Option<&'a str>,
    pub(crate) invocation_id: &'a str,
    pub(crate) space_id: &'a str,
    pub(crate) action_id: &'a str,
    pub(crate) parameters: &'a Value,
    pub(crate) now: i64,
}

pub(crate) async fn execute_immediate(
    tx: &mut Transaction<'_, Sqlite>,
    request: ImmediateActionRequest<'_>,
) -> Result<Value, AppError> {
    match request.action_id {
        "create-note" | "create-markdown" | "commit-knowledge-changes" | "edit-space-structure" => {
            execute_write_action(tx, request).await
        }
        "list-space-contents"
        | "search-knowledge"
        | "read-node"
        | "read-node-chunks"
        | "list-object-types"
        | "list-relations" => execute_read_action(tx, request).await,
        _ => Err("action-executor-unavailable: immediate Action has no registered handler".into()),
    }
}

async fn execute_write_action(
    tx: &mut Transaction<'_, Sqlite>,
    request: ImmediateActionRequest<'_>,
) -> Result<Value, AppError> {
    let ImmediateActionRequest {
        principal,
        run_id,
        invocation_id,
        space_id,
        action_id,
        parameters,
        now,
    } = request;
    match action_id {
        "create-note" => {
            crate::spaces::nodes::create_note_action::execute(
                tx, principal, space_id, parameters, now,
            )
            .await
        }
        "create-markdown" => {
            crate::spaces::nodes::create_markdown_action::execute(
                tx, principal, space_id, parameters, now,
            )
            .await
        }
        "commit-knowledge-changes" => {
            crate::spaces::knowledge::action::commit_changes(
                tx, principal, run_id, space_id, parameters, now,
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
        _ => Err("action-executor-unavailable: immediate Action has no registered handler".into()),
    }
}

async fn execute_read_action(
    tx: &mut Transaction<'_, Sqlite>,
    request: ImmediateActionRequest<'_>,
) -> Result<Value, AppError> {
    let ImmediateActionRequest {
        space_id,
        action_id,
        parameters,
        ..
    } = request;
    match action_id {
        "list-space-contents" => {
            let input: ContentsParameters = parse_parameters(parameters)?;
            let page = knowledge_projection::list_contents(
                tx,
                space_id,
                i64::from(input.offset),
                i64::from(input.limit),
            )
            .await?;
            Ok(serde_json::to_value(page).map_err(db::json_error)?)
        }
        "search-knowledge" => {
            let input: SearchParameters = parse_parameters(parameters)?;
            Ok(knowledge_projection::search_knowledge(
                tx,
                space_id,
                &SearchFilter {
                    query: input.query,
                    limit: i64::from(input.limit),
                    node_ids: input.node_ids,
                    roles: input.roles,
                    mode: input.mode,
                },
            )
            .await?)
        }
        "read-node" => {
            let input: NodeParameters = parse_parameters(parameters)?;
            Ok(knowledge_projection::load_node(tx, space_id, &input.node_id).await?)
        }
        "read-node-chunks" => {
            let input: NodeChunksParameters = parse_parameters(parameters)?;
            Ok(knowledge_projection::load_node_chunks(
                tx,
                space_id,
                &input.node_id,
                ChunkReadFilter {
                    role: input.role,
                    offset: i64::from(input.offset),
                    limit: i64::from(input.limit),
                    around_chunk_id: input.around_chunk_id,
                    context_chunks: input.context_chunks.map(i64::from),
                },
            )
            .await?)
        }
        "list-object-types" => Ok(knowledge_projection::list_object_types(tx, space_id).await?),
        "list-relations" => {
            let input: RelationsParameters = parse_parameters(parameters)?;
            Ok(knowledge_projection::list_relations(
                tx,
                space_id,
                input.node_id.as_deref(),
                input.kind.as_deref(),
                i64::from(input.limit),
            )
            .await?)
        }
        _ => Err("action-executor-unavailable: immediate Action has no registered handler".into()),
    }
}

fn parse_parameters<T: DeserializeOwned>(parameters: &Value) -> Result<T, AppError> {
    serde_json::from_value(parameters.clone()).map_err(|error| db::json_error(error).into())
}

pub(crate) async fn execute_durable(
    app: &AppHandle,
    action_id: &str,
    claim: &crate::spaces::runs::AttemptClaim,
    parameters: &Value,
    inputs: &Value,
) -> Result<Value, AppError> {
    let space_id = claim.scope.require_space_id()?;
    crate::spaces::derivation::execute_action(
        app,
        action_id,
        space_id,
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
