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
    models::{CompletionOutput, ImageAnalysisRequest},
    transport::SecurityContext,
};

pub async fn analyze_image(
    app: &AppHandle,
    request: ImageAnalysisRequest,
    cancellation: &CancellationToken,
) -> AiResult<CompletionOutput> {
    validate(&request)?;
    if let crate::ai::models::ExecutionTarget::Agent { kind, .. } = &request.target {
        let text = super::agent_task::analyze_image(
            app,
            *kind,
            request.system_prompt.trim(),
            &request.instruction,
            &request.image_path,
            request.max_tokens,
            cancellation.clone(),
        )
        .await?;
        return Ok(CompletionOutput {
            text,
            provider: kind.as_str().to_owned(),
            model: crate::ai::providers::agents::registry::descriptor(*kind)
                .display_name
                .to_owned(),
        });
    }
    let ctx = SecurityContext::from_target(&request.target)?;
    let target = service_target(&ctx, request.target.model_provider_kind()?)?;

    let image = Binary::from_file(Path::new(&request.image_path))
        .map_err(|e| AiError::invalid(e.to_string()))?;
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
        result = client.exec_chat(target, chat, Some(&options)) => result,
    }
    .map_err(|e| AiError::new(AiErrorCode::ProviderRequestFailed, e.to_string()))?;

    let raw_text = response
        .first_text()
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| AiError::new(AiErrorCode::EmptyResponse, "AI provider returned no text."))?;

    let text = super::response_text::strip_markdown_fences(raw_text).to_string();

    Ok(CompletionOutput {
        text,
        provider: request.target.provider().to_string(),
        model: request.target.model().to_string(),
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
    if request.image_path.trim().is_empty() {
        return Err(AiError::invalid("Image path is required"));
    }
    if request.instruction.trim().is_empty() {
        return Err(AiError::invalid("Vision instruction is required"));
    }
    Ok(())
}
