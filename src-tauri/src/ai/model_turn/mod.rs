mod output_budget;
mod text_stream;
mod wire;

use crate::ai::{
    contracts::{ModelTurnCapability, ModelTurnRequest, ModelTurnResponse, ProviderKind},
    models::ModelExecutionTarget,
    transport::SecurityContext,
};
use crate::foundation::app_error::AppResult;

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
        _ = cancellation.cancelled() => return Err(crate::foundation::app_error::AppError::cancelled("Model task was cancelled")),
        turn = capability.execute_turn(&context, request) => turn?,
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
            return Err(crate::foundation::app_error::AppError::unsupported(
                "Unsupported ModelTurn adapter",
            ))
        }
    };
    Ok(adapter)
}

/// The genai adapter that speaks the provider's chat wire protocol; `None` when the
/// provider has no chat protocol. Text turns and visual analysis share this mapping so
/// one provider never speaks two different protocols.
pub(crate) fn genai_adapter(provider: ProviderKind) -> Option<AdapterKind> {
    match provider {
        ProviderKind::Anthropic => Some(AdapterKind::Anthropic),
        ProviderKind::Gemini => Some(AdapterKind::Gemini),
        ProviderKind::DashScope => Some(AdapterKind::Aliyun),
        ProviderKind::DeepSeek => Some(AdapterKind::DeepSeek),
        ProviderKind::Groq => Some(AdapterKind::Groq),
        ProviderKind::Ollama => Some(AdapterKind::Ollama),
        ProviderKind::Kling => None,
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
        | ProviderKind::MiniMax
        | ProviderKind::Perplexity
        | ProviderKind::Cohere
        | ProviderKind::Baidu
        | ProviderKind::Hunyuan
        | ProviderKind::Baichuan
        | ProviderKind::SenseNova
        | ProviderKind::Spark
        | ProviderKind::Cerebras
        | ProviderKind::Fireworks
        | ProviderKind::LmStudio => Some(AdapterKind::OpenAI),
    }
}

fn adapter_kind(provider: ProviderKind) -> AppResult<AdapterKind> {
    genai_adapter(provider).ok_or_else(|| {
        crate::foundation::app_error::AppError::unsupported(format!(
            "{provider} does not provide the ModelTurn text protocol"
        ))
    })
}

pub(crate) use wire::GenAiModelTurnAdapter;
pub(crate) use wire::{to_genai_options, to_genai_request};
