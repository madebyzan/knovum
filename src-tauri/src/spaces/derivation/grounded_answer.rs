use serde::Deserialize;
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::{
    ai::{
        capabilities::chat,
        models::{CompletionRequest, ExecutionTargetSelection},
        settings,
    },
    spaces::knowledge::{
        grounding::prompt,
        retrieval::hybrid::{self, HybridSearchRequest},
    },
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct GroundedAnswerRequest {
    id: String,
    space_id: String,
    inputs: Vec<AnswerInput>,
    question: String,
    target: ExecutionTargetSelection,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct AnswerInput {
    node_id: String,
    role: String,
    source_revision: String,
}

pub(super) async fn answer(
    app: &AppHandle,
    request: GroundedAnswerRequest,
) -> Result<Value, String> {
    if request.question.trim().is_empty() || request.inputs.is_empty() {
        return Err("answer-with-sources-invalid: question and sources are required".into());
    }
    if request
        .inputs
        .first()
        .is_none_or(|input| input.role != "primary")
    {
        return Err("answer-with-sources-invalid: the first source must be primary".into());
    }
    let pool = crate::spaces::db::pool(app).await?;
    let mut tx = crate::spaces::db::begin_write(pool).await?;
    for input in &request.inputs {
        super::source_validation::validate(
            &mut tx,
            &request.space_id,
            &input.node_id,
            &input.source_revision,
            super::inputs::RevisionBasis::Knowledge,
        )
        .await?;
    }
    tx.commit().await.map_err(crate::spaces::db::sql_error)?;
    let target = settings::resolve(
        app,
        request.target,
        crate::ai::contracts::ExecutionCapability::TextGeneration,
    )
    .map_err(|error| error.to_string())?;
    let (cancellation, _lease) = crate::ai::executions::acquire(request.id.clone())?;
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
            query: request.question.clone(),
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
    let sources = prompt::select_sources(retrieval.matches)?;
    if sources.is_empty() {
        return Err("answer-with-sources-empty: no indexed source supports the question".into());
    }
    let completion = chat::complete_cancellable(
        CompletionRequest {
            purpose: crate::ai::usage::UsagePurpose::Knowledge,
            target,
            system_prompt: prompt::SYSTEM.into(),
            instruction: String::new(),
            text: prompt::build(&request.question, &[], &sources)?,
            max_tokens: None,
        },
        cancellation,
    )
    .await
    .map_err(|error| error.to_string())?;
    if let Some(usage) = completion.usage.as_ref() {
        crate::spaces::runs::record_usage(pool, &request.id, usage).await?;
    }
    let citations = prompt::citations(&completion.text, &sources, &request.space_id)?;
    Ok(json!({"answer":completion.text,"citations":citations,
        "provider":completion.provider,"model":completion.model,"usage":completion.usage}))
}
