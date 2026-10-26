mod completion;
pub(crate) mod metadata;
mod persistence;
mod prompt;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{ipc::Channel, AppHandle};

use super::stage;
use crate::foundation::app_error::AppError;
use crate::spaces::derivation::source_inputs::{self, SourceInput};
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
    inputs: Vec<SourceInput>,
    purpose: String,
    target: TaskTargetSelection,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroundedDocumentOutput {
    node_id: String,
    data: Value,
}

pub(super) async fn generate_for_action(
    authority: super::action_execution::ExecutionAuthority,
    app: AppHandle,
    request: GenerateGroundedDocumentRequest,
    stages: Channel<String>,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<GroundedDocumentOutput, AppError> {
    let target = settings::resolve_task(
        &app,
        request.target.clone(),
        ExecutionCapability::TextGeneration,
    )?;
    authority.require_active(db::pool(&app).await?).await?;
    if let Err(error) = persistence::start(&app, &request, &authority).await {
        let settlement = super::failure::fail_pending(&app, &request.output_node_id).await;
        return Err(super::failure::with_settlement(error.into(), settlement));
    }
    let generated_document = execute(&app, &request, &target, &stages, cancellation.clone()).await;
    match generated_document {
        Ok(generated) => {
            if let Some(usage) = generated.usage.as_ref() {
                crate::spaces::runs::record_usage(
                    db::pool(&app).await?,
                    &request.id,
                    usage.prompt_tokens,
                    usage.completion_tokens,
                    usage.total_tokens,
                )
                .await?;
            }
            Ok(completion::complete(&app, &request, generated, &authority, cancellation).await?)
        }
        Err(error) => {
            let settlement = persistence::fail(&app, &request.id, &error.message).await;
            Err(super::failure::with_settlement(error, settlement))
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
    let sources = if request.inputs.is_empty() {
        Vec::new()
    } else {
        stage(stages, "retrieving")?;
        let sources = retrieve(app, request, &cancellation).await?;
        if sources.is_empty() {
            return Err("Selected resources contain no current indexed knowledge".into());
        }
        sources
    };
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

/// The passages of the selected resources that serve the document's purpose.
async fn retrieve(
    app: &AppHandle,
    request: &GenerateGroundedDocumentRequest,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<Vec<Value>, AppError> {
    crate::spaces::knowledge::retrieval::rebuild::prepare_space_index(
        app,
        &request.space_id,
        Some(source_inputs::node_ids(&request.inputs)),
        &request.id,
        cancellation,
    )
    .await?;
    let retrieval = hybrid::search(
        app,
        HybridSearchRequest {
            semantic: true,
            space_id: Some(request.space_id.clone()),
            query: request.purpose.clone(),
            node_ids: Some(source_inputs::node_ids(&request.inputs)),
        },
        cancellation,
    )
    .await?;
    Ok(grounding::prompt::select_sources(retrieval.matches)?)
}

struct GeneratedDocument {
    text: String,
    execution_source: ExecutionSource,
    usage: Option<crate::ai::contracts::TokenUsage>,
    citations: Value,
}
