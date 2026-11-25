import type { AiModelDefinition } from "../../../../shared/contracts/generated/AiModelDefinition";
import type { ExecutionCapability } from "../../../../shared/contracts/generated/ExecutionCapability";
import type { ProviderDescriptor } from "../../../../shared/contracts/generated/ProviderDescriptor";
import type { ProviderKind } from "../../../../shared/contracts/generated/ProviderKind";

interface StarterModelSpec {
  readonly model: string;
  readonly name: string;
  readonly vision?: boolean;
}

const STARTER_MODEL_BY_PROVIDER: Partial<Record<ProviderKind, StarterModelSpec>> = {
  openai: { model: "gpt-4.1", name: "GPT-4.1", vision: true },
  anthropic: { model: "claude-sonnet-4-5", name: "Claude Sonnet 4.5", vision: true },
  gemini: { model: "gemini-2.5-flash", name: "Gemini 2.5 Flash", vision: true },
  deepseek: { model: "deepseek-chat", name: "DeepSeek Chat" },
  openrouter: { model: "anthropic/claude-sonnet-4.5", name: "Claude Sonnet 4.5", vision: true },
  qwen: { model: "qwen-plus", name: "Qwen Plus", vision: true },
  zhipu: { model: "glm-4.5", name: "GLM-4.5" },
  siliconflow: { model: "deepseek-ai/DeepSeek-V3", name: "DeepSeek V3" },
  moonshot: { model: "kimi-k2-0905-preview", name: "Kimi K2" },
  grok: { model: "grok-3", name: "Grok 3" },
  groq: { model: "llama-3.3-70b-versatile", name: "Llama 3.3 70B" },
  mistral: { model: "mistral-large-latest", name: "Mistral Large" },
  doubao: { model: "doubao-1-5-pro-32k-250115", name: "Doubao 1.5 Pro" },
  minimax: { model: "MiniMax-M1", name: "MiniMax M1" },
};

export function starterModelsForProvider(provider: ProviderDescriptor): AiModelDefinition[] {
  const spec = STARTER_MODEL_BY_PROVIDER[provider.id];
  if (!spec || !provider.capabilities.includes("text-generation")) return [];
  const capabilities: ExecutionCapability[] = ["text-generation"];
  if (spec.vision && provider.capabilities.includes("vision-understanding")) {
    capabilities.push("vision-understanding");
  }
  return [
    {
      id: crypto.randomUUID(),
      name: spec.name,
      model: spec.model,
      capabilities,
      builtinAgent: provider.builtinAgentProtocol !== null,
    },
  ];
}

export function buildOllamaDetectedModels(
  names: readonly string[],
  provider: ProviderDescriptor,
): AiModelDefinition[] {
  if (!provider.capabilities.includes("text-generation")) return [];
  return names.slice(0, 8).map((name) => ({
    id: crypto.randomUUID(),
    name,
    model: name,
    capabilities: ["text-generation"],
    builtinAgent: provider.builtinAgentProtocol !== null,
  }));
}
