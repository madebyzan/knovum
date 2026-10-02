mod completion;
pub(crate) mod metadata;
mod persistence;
mod prompt;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{ipc::Channel, AppHandle};

use super::stage;
use crate::app_error::AppError;
use crate::{
    ai::{
        contracts::ExecutionCapability,
        settings,
        tasks::{
            completion as task_completion, ExecutionSource, TaskExecutionTarget,
            TaskTargetSelection,
        },
    },
    spaces::{
        db,
        knowledge::{
            grounding,
            retrieval::hybrid::{self, HybridSearchRequest},
        },
    },
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct GenerateGroundedDocumentRequest {
    id: String,
    space_id: String,
    output_node_id: String,
    inputs: Vec<DocumentInput>,
    purpose: String,
    target: TaskTargetSelection,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DocumentInput {
    node_id: String,
    role: String,
    source_revision: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundedDocumentOutput {
    node_id: String,
    data: Value,
    relations: Vec<super::DerivationRelation>,
}

pub(super) async fn generate_for_action(
    authority: super::action_execution::ExecutionAuthority,
    app: AppHandle,
    request: GenerateGroundedDocumentRequest,
    stages: Channel<String>,
) -> Result<GroundedDocumentOutput, AppError> {
    let target = settings::resolve_task(
        &app,
        request.target.clone(),
        ExecutionCapability::TextGeneration,
    )?;
    let (cancellation, _lease) = crate::ai::executions::acquire(request.id.clone())?;
    authority.require_active(db::pool(&app).await?).await?;
    if let Err(error) = persistence::start(&app, &request, &authority).await {
        super::failure::fail_pending(
            &app,
            &request.space_id,
            &request.output_node_id,
            super::failure::PendingNodeState::Derivation,
        )
        .await?;
        return Err(error.into());
    }
    let generated_document = execute(&app, &request, &target, &stages, cancellation.clone()).await;
    match generated_document {
        Ok(generated) => {
            if let Some(usage) = generated.usage.as_ref() {
                crate::spaces::runs::record_usage(db::pool(&app).await?, &request.id, usage)
                    .await?;
            }
            Ok(completion::complete(&app, &request, generated, &authority, &cancellation).await?)
        }
        Err(error) => {
            persistence::fail(&app, &request.id, &error.message).await?;
            Err(error)
        }
    }
}

async fn execute(
    app: &AppHandle,
    request: &GenerateGroundedDocumentRequest,
    target: &TaskExecutionTarget,
    stages: &Channel<String>,
    cancellation: tokio_util::sync::CancellationToken,
) -> Result<GeneratedDocument, AppError> {
    stage(stages, "retrieving")?;
    crate::spaces::knowledge::retrieval::rebuild::prepare_space_index(
        app,
        &request.space_id,
        Some(
            request
                .inputs
                .iter()
                .map(|input| input.node_id.clone())
                .collect(),
        ),
        &request.id,
        &cancellation,
    )
    .await?;
    let retrieval = hybrid::search(
        app,
        HybridSearchRequest {
            semantic: true,
            space_id: Some(request.space_id.clone()),
            query: request.purpose.clone(),
            node_ids: Some(
                request
                    .inputs
                    .iter()
                    .map(|input| input.node_id.clone())
                    .collect(),
            ),
        },
        &cancellation,
    )
    .await?;
    let sources = grounding::prompt::select_sources(retrieval.matches)?;
    if sources.is_empty() {
        return Err("Selected resources contain no current indexed knowledge".into());
    }
    stage(stages, "generating")?;
    let completion = task_completion::complete(
        app,
        target.clone(),
        prompt::request(&request.purpose, &sources)?,
        cancellation,
    )
    .await?;
    let citations = grounding::prompt::citations(&completion.text, &sources, &request.space_id)?;
    Ok(GeneratedDocument {
        text: completion.text,
        execution_source: completion.execution_source,
        usage: completion.usage,
        citations,
    })
}

struct GeneratedDocument {
    text: String,
    execution_source: ExecutionSource,
    usage: Option<crate::ai::contracts::TokenUsage>,
    citations: Value,
}
