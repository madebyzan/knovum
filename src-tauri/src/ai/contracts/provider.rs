use serde::{Deserialize, Serialize};
use std::fmt;

use super::media::{ImageGenerationCapability, VideoGenerationCapability};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS, schemars::JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    #[serde(rename = "openai")]
    OpenAi,
    #[serde(rename = "openai-compatible")]
    OpenAiCompatible,
    #[serde(rename = "openrouter")]
    OpenRouter,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "gemini")]
    Gemini,
    #[serde(rename = "qwen")]
    DashScope,
    #[serde(rename = "deepseek")]
    DeepSeek,
    #[serde(rename = "zhipu")]
    Zhipu,
    #[serde(rename = "siliconflow")]
    SiliconFlow,
    #[serde(rename = "kling")]
    Kling,
    #[serde(rename = "grok")]
    Grok,
    #[serde(rename = "groq")]
    Groq,
    #[serde(rename = "mistral")]
    Mistral,
    #[serde(rename = "together")]
    Together,
    #[serde(rename = "stepfun")]
    StepFun,
    #[serde(rename = "moonshot")]
    Moonshot,
    #[serde(rename = "doubao")]
    Doubao,
    #[serde(rename = "minimax")]
    MiniMax,
    #[serde(rename = "ollama")]
    Ollama,
    #[serde(rename = "perplexity")]
    Perplexity,
    #[serde(rename = "cohere")]
    Cohere,
    #[serde(rename = "baidu")]
    Baidu,
    #[serde(rename = "hunyuan")]
    Hunyuan,
    #[serde(rename = "baichuan")]
    Baichuan,
    #[serde(rename = "sensenova")]
    SenseNova,
    #[serde(rename = "spark")]
    Spark,
    #[serde(rename = "cerebras")]
    Cerebras,
    #[serde(rename = "fireworks")]
    Fireworks,
    #[serde(rename = "lmstudio")]
    LmStudio,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS, schemars::JsonSchema,
)]
pub enum ExecutionCapability {
    #[serde(rename = "text-generation")]
    TextGeneration,
    #[serde(rename = "vision-understanding")]
    VisionUnderstanding,
    #[serde(rename = "image-generation")]
    ImageGeneration,
    #[serde(rename = "video-generation")]
    VideoGeneration,
    #[serde(rename = "embedding")]
    Embedding,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ts_rs::TS, schemars::JsonSchema,
)]
pub enum RoutingPurpose {
    #[serde(rename = "writing")]
    Writing,
    #[serde(rename = "knowledge")]
    Knowledge,
    #[serde(rename = "visual-understanding")]
    VisualUnderstanding,
    #[serde(rename = "translation")]
    Translation,
    #[serde(rename = "image-generation")]
    ImageGeneration,
    #[serde(rename = "video-generation")]
    VideoGeneration,
}

impl RoutingPurpose {
    pub const fn execution_capability(self) -> ExecutionCapability {
        match self {
            Self::Writing | Self::Knowledge | Self::Translation => {
                ExecutionCapability::TextGeneration
            }
            Self::VisualUnderstanding => ExecutionCapability::VisionUnderstanding,
            Self::ImageGeneration => ExecutionCapability::ImageGeneration,
            Self::VideoGeneration => ExecutionCapability::VideoGeneration,
        }
    }
}

impl ProviderKind {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "openai" => Some(Self::OpenAi),
            "openai-compatible" => Some(Self::OpenAiCompatible),
            "openrouter" => Some(Self::OpenRouter),
            "anthropic" => Some(Self::Anthropic),
            "gemini" => Some(Self::Gemini),
            "qwen" => Some(Self::DashScope),
            "deepseek" => Some(Self::DeepSeek),
            "zhipu" => Some(Self::Zhipu),
            "siliconflow" => Some(Self::SiliconFlow),
            "kling" => Some(Self::Kling),
            "grok" => Some(Self::Grok),
            "groq" => Some(Self::Groq),
            "mistral" => Some(Self::Mistral),
            "together" => Some(Self::Together),
            "stepfun" => Some(Self::StepFun),
            "moonshot" => Some(Self::Moonshot),
            "doubao" => Some(Self::Doubao),
            "minimax" => Some(Self::MiniMax),
            "ollama" => Some(Self::Ollama),
            "perplexity" => Some(Self::Perplexity),
            "cohere" => Some(Self::Cohere),
            "baidu" => Some(Self::Baidu),
            "hunyuan" => Some(Self::Hunyuan),
            "baichuan" => Some(Self::Baichuan),
            "sensenova" => Some(Self::SenseNova),
            "spark" => Some(Self::Spark),
            "cerebras" => Some(Self::Cerebras),
            "fireworks" => Some(Self::Fireworks),
            "lmstudio" => Some(Self::LmStudio),
            _ => None,
        }
    }

    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::OpenAi => "openai",
            Self::OpenAiCompatible => "openai-compatible",
            Self::OpenRouter => "openrouter",
            Self::Anthropic => "anthropic",
            Self::Gemini => "gemini",
            Self::DashScope => "qwen",
            Self::DeepSeek => "deepseek",
            Self::Zhipu => "zhipu",
            Self::SiliconFlow => "siliconflow",
            Self::Kling => "kling",
            Self::Grok => "grok",
            Self::Groq => "groq",
            Self::Mistral => "mistral",
            Self::Together => "together",
            Self::StepFun => "stepfun",
            Self::Moonshot => "moonshot",
            Self::Doubao => "doubao",
            Self::MiniMax => "minimax",
            Self::Ollama => "ollama",
            Self::Perplexity => "perplexity",
            Self::Cohere => "cohere",
            Self::Baidu => "baidu",
            Self::Hunyuan => "hunyuan",
            Self::Baichuan => "baichuan",
            Self::SenseNova => "sensenova",
            Self::Spark => "spark",
            Self::Cerebras => "cerebras",
            Self::Fireworks => "fireworks",
            Self::LmStudio => "lmstudio",
        }
    }
}

impl fmt::Display for ProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub trait AiProvider: Send + Sync {
    fn as_image_generation(&self) -> Option<&dyn ImageGenerationCapability> {
        None
    }

    fn as_video_generation(&self) -> Option<&dyn VideoGenerationCapability> {
        None
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema, ts_rs::TS)]
#[serde(rename_all = "snake_case")]
pub enum ChatModelProtocol {
    OpenAiChat,
    OpenAiResponses,
    Anthropic,
    Gemini,
    Ollama,
    DeepSeek,
    Groq,
}
