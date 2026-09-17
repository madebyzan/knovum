use serde::Serialize;

use crate::ai::contracts::{ExecutionCapability, ProviderKind};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderDescriptor {
    pub id: &'static str,
    pub display_name: &'static str,
    pub default_base_url: &'static str,
    pub capabilities: &'static [ExecutionCapability],
}

const TEXT: &[ExecutionCapability] = &[ExecutionCapability::TextGeneration];
const TEXT_VISION: &[ExecutionCapability] = &[
    ExecutionCapability::TextGeneration,
    ExecutionCapability::VisionUnderstanding,
];
const TEXT_VISION_IMAGE: &[ExecutionCapability] = &[
    ExecutionCapability::TextGeneration,
    ExecutionCapability::VisionUnderstanding,
    ExecutionCapability::ImageGeneration,
];
const TEXT_VISION_MEDIA: &[ExecutionCapability] = &[
    ExecutionCapability::TextGeneration,
    ExecutionCapability::VisionUnderstanding,
    ExecutionCapability::ImageGeneration,
    ExecutionCapability::VideoGeneration,
];
const MEDIA: &[ExecutionCapability] = &[
    ExecutionCapability::ImageGeneration,
    ExecutionCapability::VideoGeneration,
];

const PROVIDERS: &[ProviderDescriptor] = &[
    define_provider(
        ProviderKind::OpenAi,
        "OpenAI",
        "https://api.openai.com/v1/",
        TEXT_VISION_IMAGE,
    ),
    define_provider(
        ProviderKind::OpenAiCompatible,
        "OpenAI-compatible",
        "",
        TEXT_VISION_IMAGE,
    ),
    define_provider(
        ProviderKind::Anthropic,
        "Anthropic",
        "https://api.anthropic.com/",
        TEXT_VISION,
    ),
    define_provider(
        ProviderKind::Gemini,
        "Google Gemini",
        "https://generativelanguage.googleapis.com/v1beta/",
        TEXT_VISION_MEDIA,
    ),
    define_provider(
        ProviderKind::OpenRouter,
        "OpenRouter",
        "https://openrouter.ai/api/v1/",
        TEXT_VISION_MEDIA,
    ),
    define_provider(
        ProviderKind::DashScope,
        "Alibaba Cloud Model Studio",
        "https://dashscope.aliyuncs.com/compatible-mode/v1/",
        TEXT_VISION_MEDIA,
    ),
    define_provider(
        ProviderKind::DeepSeek,
        "DeepSeek",
        "https://api.deepseek.com/",
        TEXT,
    ),
    define_provider(
        ProviderKind::Grok,
        "xAI",
        "https://api.x.ai/v1/",
        TEXT_VISION_IMAGE,
    ),
    define_provider(
        ProviderKind::Groq,
        "Groq",
        "https://api.groq.com/openai/v1/",
        TEXT_VISION,
    ),
    define_provider(
        ProviderKind::Mistral,
        "Mistral AI",
        "https://api.mistral.ai/v1/",
        TEXT_VISION,
    ),
    define_provider(
        ProviderKind::Together,
        "Together AI",
        "https://api.together.xyz/v1/",
        TEXT_VISION_MEDIA,
    ),
    define_provider(
        ProviderKind::StepFun,
        "StepFun",
        "https://api.stepfun.com/v1/",
        TEXT_VISION_MEDIA,
    ),
    define_provider(
        ProviderKind::Moonshot,
        "Moonshot AI",
        "https://api.moonshot.cn/v1/",
        TEXT_VISION,
    ),
    define_provider(
        ProviderKind::Doubao,
        "ByteDance ModelArk",
        "https://ark.cn-beijing.volces.com/api/v3/",
        TEXT_VISION_MEDIA,
    ),
    define_provider(
        ProviderKind::Zhipu,
        "Zhipu AI",
        "https://open.bigmodel.cn/api/paas/v4/",
        TEXT_VISION_MEDIA,
    ),
    define_provider(
        ProviderKind::SiliconFlow,
        "SiliconFlow",
        "https://api.siliconflow.cn/v1/",
        TEXT_VISION_MEDIA,
    ),
    define_provider(
        ProviderKind::MiniMax,
        "MiniMax",
        "https://api.minimax.chat/v1/",
        TEXT_VISION_MEDIA,
    ),
    define_provider(
        ProviderKind::Kling,
        "Kling AI",
        "https://api.klingai.com/",
        MEDIA,
    ),
    define_provider(
        ProviderKind::Ollama,
        "Ollama",
        "http://localhost:11434/",
        TEXT_VISION,
    ),
];

const fn define_provider(
    kind: ProviderKind,
    display_name: &'static str,
    default_base_url: &'static str,
    capabilities: &'static [ExecutionCapability],
) -> ProviderDescriptor {
    ProviderDescriptor {
        id: kind.as_str(),
        display_name,
        default_base_url,
        capabilities,
    }
}

pub fn descriptors() -> &'static [ProviderDescriptor] {
    PROVIDERS
}

pub fn descriptor(kind: ProviderKind) -> Option<&'static ProviderDescriptor> {
    PROVIDERS
        .iter()
        .find(|descriptor| descriptor.id == kind.as_str())
}
