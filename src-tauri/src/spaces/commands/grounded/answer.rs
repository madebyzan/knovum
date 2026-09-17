use serde_json::Value;
use tauri::{ipc::Channel, AppHandle};
use uuid::Uuid;

use crate::{
    ai::{
        capabilities::chat,
        executions,
        models::{CompletionOutput, CompletionRequest, CompletionStreamEvent},
    },
    spaces::knowledge::{grounding::prompt, retrieval::hybrid},
};

use super::answer_context::{load_context, Context};
use super::answer_request::GroundedKnowledgeRequest;
use super::contracts::{GroundedExchangeMessageRequest, GroundedExchangeRequest};

fn send_stage(channel: &Channel<String>, stage: &str) -> Result<(), String> {
    channel
        .send(stage.to_owned())
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn answer_grounded_question(
    app: AppHandle,
    request: GroundedKnowledgeRequest,
    on_event: Channel<CompletionStreamEvent>,
    on_stage: Channel<String>,
) -> Result<Value, String> {
    let question = request.question.trim();
    if question.is_empty() {
        return Err("Grounded question is required".to_owned());
    }
    let (cancellation, _lease) = executions::acquire(request.job_id.clone())?;
    let target = crate::ai::settings::resolve(
        &app,
        request.target.clone(),
        crate::ai::contracts::ExecutionCapability::TextGeneration,
    )
    .map_err(|error| error.to_string())?;
    let context = load_context(&app, &request).await?;
    send_stage(&on_stage, "retrieving")?;
    let retrieval = hybrid::search(
        &app,
        hybrid::HybridSearchRequest {
            space_id: (context.scope != "all-spaces").then(|| request.space_id.clone()),
            query: question.to_owned(),
            node_ids: matches!(context.scope.as_str(), "selection" | "section")
                .then(|| context.node_ids.clone()),
        },
    )
    .await?;
    let sources = prompt::select_sources(retrieval.matches)?;
    if sources.is_empty() {
        return Err("No indexed knowledge supports this question".to_owned());
    }
    send_stage(&on_stage, "generating")?;
    let completion = chat::complete_stream(
        CompletionRequest {
            target,
            system_prompt: prompt::SYSTEM.to_owned(),
            instruction: String::new(),
            text: prompt::build(question, &context.history, &sources)?,
            max_tokens: None,
        },
        on_event,
        cancellation,
    )
    .await
    .map_err(|error| error.to_string())?;
    let citations = prompt::citations(&completion.text, &sources, &request.space_id)?;
    send_stage(&on_stage, "saving")?;
    let user_id = Uuid::new_v4().to_string();
    let assistant_id = Uuid::new_v4().to_string();
    let exchange = build_exchange_request(
        &request,
        &context,
        question,
        &completion,
        &citations,
        &user_id,
        &assistant_id,
    );
    let committed = super::commit_exchange(&app, exchange).await?;
    Ok(build_exchange_response(
        &committed,
        &user_id,
        &assistant_id,
        question,
        &completion,
        citations,
        retrieval.semantic.into(),
    ))
}

fn build_exchange_request(
    request: &GroundedKnowledgeRequest,
    context: &Context,
    question: &str,
    completion: &CompletionOutput,
    citations: &Value,
    user_id: &str,
    assistant_id: &str,
) -> GroundedExchangeRequest {
    GroundedExchangeRequest {
        thread_id: request.thread_id.clone(),
        new_thread: context.new_thread(question),
        space_id: request.space_id.clone(),
        user: GroundedExchangeMessageRequest {
            id: user_id.to_owned(),
            content: question.to_owned(),
            citations: Value::Array(Vec::new()),
            provider: None,
            model: None,
        },
        assistant: GroundedExchangeMessageRequest {
            id: assistant_id.to_owned(),
            content: completion.text.clone(),
            citations: citations.clone(),
            provider: Some(completion.provider.clone()),
            model: Some(completion.model.clone()),
        },
    }
}

fn build_exchange_response(
    committed: &Value,
    user_id: &str,
    assistant_id: &str,
    question: &str,
    completion: &CompletionOutput,
    citations: Value,
    semantic: Value,
) -> Value {
    serde_json::json!({
        "threadId": committed["threadId"],
        "user": {
            "id": user_id,
            "threadId": committed["threadId"],
            "role": "user",
            "content": question,
            "citations": [],
            "createdAt": committed["userCreatedAt"],
        },
        "assistant": {
            "id": assistant_id,
            "threadId": committed["threadId"],
            "role": "assistant",
            "content": completion.text,
            "citations": citations,
            "provider": completion.provider,
            "model": completion.model,
            "createdAt": committed["assistantCreatedAt"],
        },
        "semantic": semantic,
    })
}
