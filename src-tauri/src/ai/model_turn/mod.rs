mod output_budget;
mod text_stream;
mod wire;

use crate::ai::{
    contracts::{ModelTurnCapability, ModelTurnRequest, ModelTurnResponse, ProviderKind},
    models::ModelExecutionTarget,
    transport::SecurityContext,
};
use crate::app_error::AppResult;

use genai::adapter::AdapterKind;
use tokio_util::sync::CancellationToken;

static OPENAI: GenAiModelTurnAdapter = GenAiModelTurnAdapter::new(AdapterKind::OpenAI);
static ANTHROPIC: GenAiModelTurnAdapter = GenAiModelTurnAdapter::new(AdapterKind::Anthropic);
static GEMINI: GenAiModelTurnAdapter = GenAiModelTurnAdapter::new(AdapterKind::Gemini);
static ALIYUN: GenAiModelTurnAdapter = GenAiModelTurnAdapter::new(AdapterKind::Aliyun);
static DEEPSEEK: GenAiModelTurnAdapter = GenAiModelTurnAdapter::new(AdapterKind::DeepSeek);
static GROQ: GenAiModelTurnAdapter = GenAiModelTurnAdapter::new(AdapterKind::Groq);
static OLLAMA: GenAiModelTurnAdapter = GenAiModelTurnAdapter::new(AdapterKind::Ollama);

pub async fn execute(
    target: &ModelExecutionTarget,
    request: &ModelTurnRequest,
    cancellation: &CancellationToken,
) -> AppResult<ModelTurnResponse> {
    let context = SecurityContext::from_target(target).await?;
    let capability = adapter(target.model_provider_kind()?)?;
    let response = tokio::select! {
        biased;
        _ = cancellation.cancelled() => return Err(crate::app_error::AppError::cancelled("Model task was cancelled")),
        result = capability.execute_turn(&context, request) => result?,
    };
    crate::ai::usage::record_turn(request.purpose, &response).await?;
    Ok(response)
}

pub async fn stream_text(
    target: &ModelExecutionTarget,
    request: &ModelTurnRequest,
    cancellation: CancellationToken,
) -> AppResult<crate::ai::contracts::StreamFrameStream> {
    let context = SecurityContext::from_target(target).await?;
    let adapter = adapter_kind(target.model_provider_kind()?)?;
    text_stream::stream_text(adapter, context, request.clone(), cancellation).await
}

fn adapter(provider: ProviderKind) -> AppResult<&'static dyn ModelTurnCapability> {
    let adapter = match adapter_kind(provider)? {
        AdapterKind::OpenAI => &OPENAI as &dyn ModelTurnCapability,
        AdapterKind::Anthropic => &ANTHROPIC,
        AdapterKind::Gemini => &GEMINI,
        AdapterKind::Aliyun => &ALIYUN,
        AdapterKind::DeepSeek => &DEEPSEEK,
        AdapterKind::Groq => &GROQ,
        AdapterKind::Ollama => &OLLAMA,
        _ => {
            return Err(crate::app_error::AppError::unsupported(
                "Unsupported ModelTurn adapter",
            ))
        }
    };
    Ok(adapter)
}

fn adapter_kind(provider: ProviderKind) -> AppResult<AdapterKind> {
    match provider {
        ProviderKind::Anthropic => Ok(AdapterKind::Anthropic),
        ProviderKind::Gemini => Ok(AdapterKind::Gemini),
        ProviderKind::DashScope => Ok(AdapterKind::Aliyun),
        ProviderKind::DeepSeek => Ok(AdapterKind::DeepSeek),
        ProviderKind::Groq => Ok(AdapterKind::Groq),
        ProviderKind::Ollama => Ok(AdapterKind::Ollama),
        ProviderKind::Kling => Err(crate::app_error::AppError::unsupported(
            "Kling does not provide the ModelTurn text protocol",
        )),
        ProviderKind::OpenAi
        | ProviderKind::OpenAiCompatible
        | ProviderKind::OpenRouter
        | ProviderKind::Zhipu
        | ProviderKind::SiliconFlow
        | ProviderKind::Grok
        | ProviderKind::Mistral
        | ProviderKind::Together
        | ProviderKind::StepFun
        | ProviderKind::Moonshot
        | ProviderKind::Doubao
        | ProviderKind::MiniMax => Ok(AdapterKind::OpenAI),
    }
}

pub(crate) use wire::GenAiModelTurnAdapter;
pub(crate) use wire::{to_genai_options, to_genai_request};
