import type { AiModelDefinition } from "../../../../shared/contracts/generated/AiModelDefinition";
import type { ProviderDescriptor } from "../../../../shared/contracts/generated/ProviderDescriptor";

const NON_CHAT_OLLAMA_MODEL = /(?:embed|bge-|e5-|minilm|rerank|gte-)/i;
const MAX_OLLAMA_IMPORT_MODELS = 20;

export function filterOllamaChatModelNames(names: readonly string[]): string[] {
  return names.filter((name) => !NON_CHAT_OLLAMA_MODEL.test(name)).slice(0, MAX_OLLAMA_IMPORT_MODELS);
}

export function buildOllamaDetectedModels(
  names: readonly string[],
  provider: ProviderDescriptor,
): AiModelDefinition[] {
  if (!provider.capabilities.includes("text-generation")) return [];
  return filterOllamaChatModelNames(names).map((name) => ({
    id: crypto.randomUUID(),
    name,
    model: name,
    capabilities: ["text-generation"],
    builtinAgent: provider.builtinAgentProtocol !== null,
  }));
}
