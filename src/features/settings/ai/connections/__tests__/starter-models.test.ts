import { describe, expect, it } from "vitest";

import type { ProviderDescriptor } from "../../../../../shared/contracts/generated/ProviderDescriptor";
import {
  buildOllamaDetectedModels,
  filterOllamaChatModelNames,
  starterModelsForProvider,
} from "../starter-models";

const OLLAMA_PROVIDER: ProviderDescriptor = {
  id: "ollama",
  displayName: "Ollama",
  capabilities: ["text-generation"],
  defaultBaseUrl: "http://127.0.0.1:11434/v1",
  builtinAgentProtocol: "ollama",
};

describe("starter and Ollama detected models", () => {
  it("filters embedding and reranker models and honors up to 20 chat models", () => {
    const names = [
      "nomic-embed-text:latest",
      "mxbai-embed-large",
      "bge-m3:latest",
      "bge-reranker-v2-m3",
      "snowflake-arctic-embed2",
      "all-minilm:l6-v2",
      "e5-large-v2",
      "gte-qwen2",
      ...Array.from({ length: 24 }, (_, index) => `qwen3:${index + 1}b`),
    ];
    const filtered = filterOllamaChatModelNames(names);
    expect(filtered).toHaveLength(20);
    expect(filtered[0]).toBe("qwen3:1b");
    expect(filtered[19]).toBe("qwen3:20b");

    const built = buildOllamaDetectedModels(names, OLLAMA_PROVIDER);
    expect(built).toHaveLength(20);
    expect(built.every((model) => model.capabilities.includes("text-generation"))).toBe(true);
  });

  it("adds vision capability to starter models when supported by provider", () => {
    const models = starterModelsForProvider({
      id: "openai",
      displayName: "OpenAI",
      capabilities: ["text-generation", "vision-understanding"],
      defaultBaseUrl: "https://api.openai.com/v1",
      builtinAgentProtocol: "open_ai_responses",
    });
    expect(models).toHaveLength(1);
    expect(models[0]?.capabilities).toEqual(["text-generation", "vision-understanding"]);
  });
});
