mod completion;
pub(crate) mod metadata;
mod persistence;
mod prompt;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{ipc::Channel, AppHandle};

use super::stage;
use crate::{
    ai::{
        capabilities::chat,
        contracts::ExecutionCapability,
        models::{ExecutionTarget, ExecutionTargetSelection},
        settings,
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
pub struct CreateGroundedDocumentRequest {
    id: String,
    space_id: String,
    output_node_id: String,
    inputs: Vec<DocumentInput>,
    purpose: String,
    target: ExecutionTargetSelection,
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

pub async fn create(
    app: AppHandle,
    request: CreateGroundedDocumentRequest,
    stages: Channel<String>,
) -> Result<GroundedDocumentOutput, String> {
    let target = settings::resolve(
        &app,
        request.target.clone(),
        ExecutionCapability::TextGeneration,
    )
    .map_err(|error| error.to_string())?;
    let (cancellation, _lease) =
        crate::ai::executions::acquire(request.id.clone()).map_err(|error| error.to_string())?;
    if let Err(error) = persistence::start(&app, &request).await {
        super::failure::fail_pending(
            &app,
            &request.space_id,
            &request.output_node_id,
            super::failure::PendingNodeState::Derivation,
        )
        .await?;
        return Err(error);
    }
    let execution = execute(&app, &request, &target, &stages, cancellation.clone());
    let result = tokio::select! {
        _ = cancellation.cancelled() => Err("Grounded document creation was cancelled".to_owned()),
        result = execution => result,
    };
    match result {
        Ok(generated) => {
            if let Some(usage) = generated.usage.as_ref() {
                crate::spaces::runs::record_usage(db::pool(&app).await?, &request.id, usage)
                    .await?;
            }
            completion::complete(&app, &request, generated).await
        }
        Err(error) => {
            persistence::fail(&app, &request.id, &error).await?;
            Err(error)
        }
    }
}

async fn execute(
    app: &AppHandle,
    request: &CreateGroundedDocumentRequest,
    target: &ExecutionTarget,
    stages: &Channel<String>,
    cancellation: tokio_util::sync::CancellationToken,
) -> Result<GeneratedDocument, String> {
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
    let completion = chat::complete_cancellable(
        prompt::request(target, &request.purpose, &sources)?,
        cancellation,
    )
    .await
    .map_err(|error| error.to_string())?;
    let citations = grounding::prompt::citations(&completion.text, &sources, &request.space_id)?;
    Ok(GeneratedDocument {
        text: completion.text,
        provider: completion.provider,
        model: completion.model,
        usage: completion.usage,
        citations,
    })
}

struct GeneratedDocument {
    text: String,
    provider: String,
    model: String,
    usage: Option<crate::ai::contracts::TokenUsage>,
    citations: Value,
}
