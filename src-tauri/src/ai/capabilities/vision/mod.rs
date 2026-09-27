use base64::Engine;
use genai::{
    adapter::AdapterKind,
    chat::{
        Binary, ChatMessage, ChatOptions, ChatRequest, ChatResponseFormat, ContentPart,
        MessageContent,
    },
    resolver::{AuthData, Endpoint},
    Client, ModelIden, ServiceTarget,
};
use std::path::Path;
use tauri::AppHandle;
use tokio_util::sync::CancellationToken;

use crate::ai::{
    contracts::ProviderKind,
    error::{AiError, AiErrorCode, AiResult},
    models::{CompletionOutput, ImageAnalysisRequest, ImageAnalysisSource},
    transport::SecurityContext,
};

pub async fn analyze_image(
    _app: &AppHandle,
    request: ImageAnalysisRequest,
    cancellation: &CancellationToken,
) -> AiResult<CompletionOutput> {
    validate(&request)?;
    let ctx = SecurityContext::from_target(&request.target).await?;
    let target = service_target(&ctx, request.target.model_provider_kind()?)?;

    let image = match &request.source {
        ImageAnalysisSource::FilePath(path) => {
            Binary::from_file(Path::new(path)).map_err(|e| AiError::invalid(e.to_string()))?
        }
        ImageAnalysisSource::Png(bytes) => Binary::from_base64(
            "image/png",
            base64::engine::general_purpose::STANDARD.encode(bytes),
            None,
        ),
    };
    if !image.is_image() {
        return Err(AiError::invalid("Vision analysis requires an image file"));
    }

    let content = MessageContent::from_parts(vec![
        ContentPart::Text(request.instruction.clone()),
        ContentPart::Binary(image),
    ]);
    let chat = ChatRequest::from_messages(vec![ChatMessage::user(content)])
        .with_system(request.system_prompt.trim());
    let mut options = ChatOptions::default();
    options = options.with_temperature(ctx.temperature);
    if matches!(
        request.target.provider(),
        "openai"
            | "openai-compatible"
            | "deepseek"
            | "groq"
            | "openrouter"
            | "siliconflow"
            | "zhipu"
            | "moonshot"
    ) {
        options = options.with_response_format(ChatResponseFormat::JsonMode);
    }
    if let Some(max) = request.max_tokens {
        options = options.with_max_tokens(max);
    }

    let client = Client::default();
    let response = tokio::select! {
        _ = cancellation.cancelled() => {
            return Err(AiError::cancelled("Visual analysis was cancelled"));
        }
        vision_completion = client.exec_chat(target, chat, Some(&options)) => vision_completion,
    }
    .map_err(|e| AiError::new(AiErrorCode::ProviderRequestFailed, e.to_string()))?;

    let raw_text = response
        .first_text()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| AiError::new(AiErrorCode::EmptyResponse, "AI provider returned no text."))?;

    let text = super::response_text::strip_markdown_fences(raw_text).to_string();
    let usage = crate::ai::usage::from_provider_usage(response.usage);
    if let Some(usage) = usage.as_ref() {
        crate::ai::usage::record(
            crate::ai::usage::UsagePurpose::VisualAnalysis,
            request.target.provider(),
            request.target.model(),
            usage,
        )
        .await?;
    }

    Ok(CompletionOutput {
        text,
        provider: request.target.provider().to_string(),
        model: request.target.model().to_string(),
        usage,
    })
}

fn service_target(ctx: &SecurityContext, kind: ProviderKind) -> AiResult<ServiceTarget> {
    let adapter = match kind {
        ProviderKind::OpenAi
        | ProviderKind::OpenAiCompatible
        | ProviderKind::OpenRouter
        | ProviderKind::DeepSeek
        | ProviderKind::Grok
        | ProviderKind::Groq
        | ProviderKind::Mistral
        | ProviderKind::Together
        | ProviderKind::StepFun
        | ProviderKind::Moonshot
        | ProviderKind::Doubao
        | ProviderKind::MiniMax
        | ProviderKind::Zhipu
        | ProviderKind::SiliconFlow => AdapterKind::OpenAI,

        ProviderKind::Anthropic => AdapterKind::Anthropic,
        ProviderKind::Gemini => AdapterKind::Gemini,
        ProviderKind::DashScope => AdapterKind::Aliyun,
        ProviderKind::Ollama => AdapterKind::Ollama,
        _ => {
            return Err(AiError::unsupported(
                "Vision is not supported on this provider",
            ))
        }
    };

    let endpoint = format!("{}/", ctx.endpoint.trim_end_matches('/'));
    let auth = match &ctx.api_key {
        Some(k) => AuthData::from_single(k.clone()),
        None => AuthData::None,
    };
    Ok(ServiceTarget {
        endpoint: Endpoint::from_owned(endpoint),
        auth,
        model: ModelIden::new(adapter, ctx.model.clone()),
    })
}

fn validate(request: &ImageAnalysisRequest) -> AiResult<()> {
    match &request.source {
        ImageAnalysisSource::FilePath(path) if path.trim().is_empty() => {
            return Err(AiError::invalid("Image path is required"))
        }
        ImageAnalysisSource::Png(bytes) if bytes.is_empty() => {
            return Err(AiError::invalid("Image PNG is required"))
        }
        _ => {}
    }
    if request.instruction.trim().is_empty() {
        return Err(AiError::invalid("Vision instruction is required"));
    }
    Ok(())
}
