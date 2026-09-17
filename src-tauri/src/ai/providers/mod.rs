use std::sync::OnceLock;

pub mod agents;
pub mod anthropic;
pub mod dashscope;
pub mod deepseek;
pub mod doubao;
pub mod gemini;
pub mod grok;
pub mod kling;
pub mod local_candle;
pub mod minimax;
pub mod moonshot;
pub mod ollama;
pub mod openai;
pub mod openrouter;
pub mod registry;
pub mod siliconflow;
pub mod stepfun;
pub mod together;
pub mod zhipu;

use anthropic::AnthropicProvider;
use dashscope::DashScopeProvider;
use deepseek::DeepSeekProvider;
use doubao::DoubaoProvider;
use gemini::GeminiProvider;
use grok::GrokProvider;
use kling::KlingProvider;
use minimax::MiniMaxProvider;
use moonshot::MoonshotProvider;
use ollama::OllamaProvider;
use openai::OpenAiProvider;
use openrouter::OpenRouterProvider;
use siliconflow::SiliconFlowProvider;
use stepfun::StepFunProvider;
use together::TogetherProvider;
use zhipu::ZhipuProvider;

use super::contracts::{AiProvider, ProviderKind};

pub fn resolve_provider(kind: ProviderKind) -> &'static dyn AiProvider {
    static OPENAI: OnceLock<OpenAiProvider> = OnceLock::new();
    static OPENAI_COMPATIBLE: OnceLock<OpenAiProvider> = OnceLock::new();
    static OPENROUTER: OnceLock<OpenRouterProvider> = OnceLock::new();
    static ANTHROPIC: OnceLock<AnthropicProvider> = OnceLock::new();
    static GEMINI: OnceLock<GeminiProvider> = OnceLock::new();
    static DASHSCOPE: OnceLock<DashScopeProvider> = OnceLock::new();
    static DEEPSEEK: OnceLock<DeepSeekProvider> = OnceLock::new();
    static ZHIPU: OnceLock<ZhipuProvider> = OnceLock::new();
    static SILICONFLOW: OnceLock<SiliconFlowProvider> = OnceLock::new();
    static KLING: OnceLock<KlingProvider> = OnceLock::new();
    static GROK: OnceLock<GrokProvider> = OnceLock::new();
    static GROQ: OnceLock<OpenAiProvider> = OnceLock::new();
    static MISTRAL: OnceLock<OpenAiProvider> = OnceLock::new();
    static TOGETHER: OnceLock<TogetherProvider> = OnceLock::new();
    static STEPFUN: OnceLock<StepFunProvider> = OnceLock::new();
    static MOONSHOT: OnceLock<MoonshotProvider> = OnceLock::new();
    static DOUBAO: OnceLock<DoubaoProvider> = OnceLock::new();
    static MINIMAX: OnceLock<MiniMaxProvider> = OnceLock::new();
    static OLLAMA: OnceLock<OllamaProvider> = OnceLock::new();

    match kind {
        ProviderKind::OpenAi => OPENAI.get_or_init(|| OpenAiProvider::new(ProviderKind::OpenAi)),
        ProviderKind::OpenAiCompatible => {
            OPENAI_COMPATIBLE.get_or_init(|| OpenAiProvider::new(ProviderKind::OpenAiCompatible))
        }
        ProviderKind::OpenRouter => OPENROUTER.get_or_init(OpenRouterProvider::new),
        ProviderKind::Anthropic => ANTHROPIC.get_or_init(AnthropicProvider::new),
        ProviderKind::Gemini => GEMINI.get_or_init(GeminiProvider::new),
        ProviderKind::DashScope => DASHSCOPE.get_or_init(DashScopeProvider::new),
        ProviderKind::DeepSeek => DEEPSEEK.get_or_init(DeepSeekProvider::new),
        ProviderKind::Zhipu => ZHIPU.get_or_init(ZhipuProvider::new),
        ProviderKind::SiliconFlow => SILICONFLOW.get_or_init(SiliconFlowProvider::new),
        ProviderKind::Kling => KLING.get_or_init(KlingProvider::new),
        ProviderKind::Grok => GROK.get_or_init(GrokProvider::new),
        ProviderKind::Groq => GROQ.get_or_init(|| OpenAiProvider::new(ProviderKind::Groq)),
        ProviderKind::Mistral => MISTRAL.get_or_init(|| OpenAiProvider::new(ProviderKind::Mistral)),
        ProviderKind::Together => TOGETHER.get_or_init(TogetherProvider::new),
        ProviderKind::StepFun => STEPFUN.get_or_init(StepFunProvider::new),
        ProviderKind::Moonshot => MOONSHOT.get_or_init(MoonshotProvider::new),
        ProviderKind::Doubao => DOUBAO.get_or_init(DoubaoProvider::new),
        ProviderKind::MiniMax => MINIMAX.get_or_init(MiniMaxProvider::new),
        ProviderKind::Ollama => OLLAMA.get_or_init(OllamaProvider::new),
    }
}
