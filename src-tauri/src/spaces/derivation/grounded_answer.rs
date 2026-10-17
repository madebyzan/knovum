use serde::Deserialize;
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::foundation::app_error::AppError;
use crate::spaces::derivation::source_inputs::{self, SourceInput};
use crate::{
    ai::{
        settings,
        tasks::{
            completion::{self, TaskInput},
            TaskTargetSelection,
        },
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
    inputs: Vec<SourceInput>,
    question: String,
    target: TaskTargetSelection,
}

pub(super) async fn answer(
    app: &AppHandle,
    request: GroundedAnswerRequest,
    authority: &super::action_execution::ExecutionAuthority,
    cancellation: &tokio_util::sync::CancellationToken,
) -> Result<Value, AppError> {
    if request.question.trim().is_empty() {
        return Err(AppError::new(
            "answer_with_sources_invalid",
            "question and sources are required",
        ));
    }
    source_inputs::require_single_primary(&request.inputs)
        .map_err(|error| AppError::new("answer_with_sources_invalid", error))?;
    let pool = crate::spaces::db::pool(app).await?;
    let mut tx = crate::spaces::db::begin_write(pool).await?;
    source_inputs::pin(&mut tx, &request.space_id, &request.inputs).await?;
    tx.commit().await.map_err(crate::spaces::db::sql_error)?;
    let target = settings::resolve_task(
        app,
        request.target,
        crate::ai::contracts::ExecutionCapability::TextGeneration,
    )?;
    authority.require_active(pool).await?;
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
            query: request.question.clone(),
            node_ids: Some(source_inputs::node_ids(&request.inputs)),
        },
        cancellation,
    )
    .await?;
    let sources = prompt::select_sources(retrieval.matches)?;
    if sources.is_empty() {
        return Err(AppError::new(
            "answer_with_sources_empty",
            "no indexed source supports the question",
        ));
    }
    let completion = completion::complete(
        app,
        target,
        TaskInput {
            purpose: crate::ai::usage::UsagePurpose::Knowledge,
            system_prompt: prompt::SYSTEM.into(),
            instruction: String::new(),
            text: prompt::build(&request.question, &[], &sources)?,
            max_tokens: None,
            image: None,
        },
        cancellation.clone(),
    )
    .await?;
    if let Some(usage) = completion.usage.as_ref() {
        crate::spaces::runs::record_usage(
            pool,
            &request.id,
            usage.prompt_tokens,
            usage.completion_tokens,
            usage.total_tokens,
        )
        .await?;
    }
    let citations = prompt::citations(&completion.text, &sources, &request.space_id)?;
    Ok(json!({"answer":completion.text,"citations":citations,
        "executionSource":completion.execution_source,"usage":completion.usage}))
}
