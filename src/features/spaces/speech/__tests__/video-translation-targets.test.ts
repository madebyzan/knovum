import { describe, expect, it } from "vitest";

import {
  parseTranslationTerms,
  persistedTranslationTarget,
  sameTranslationTarget,
} from "../video-translation-targets";

describe("video-translation-targets", () => {
  it("parses terms separated by comma, semicolon, and newline", () => {
    expect(parseTranslationTerms("Knovum, DeepSeek；Alibaba\nGoogle,  Tencent ")).toEqual([
      "Knovum",
      "DeepSeek",
      "Alibaba",
      "Google",
      "Tencent",
    ]);
  });

  it("handles empty or whitespace-only terms input", () => {
    expect(parseTranslationTerms("")).toEqual([]);
    expect(parseTranslationTerms("  , ; \n  ")).toEqual([]);
  });

  it("matches identical translation targets", () => {
    const target1 = { type: "model" as const, connectionId: "c1", modelId: "m1" };
    const target2 = { type: "model" as const, connectionId: "c1", modelId: "m1" };
    const target3 = { type: "model" as const, connectionId: "c1", modelId: "m2" };
    expect(sameTranslationTarget(target1, target2)).toBe(true);
    expect(sameTranslationTarget(target1, target3)).toBe(false);
  });

  it("resolves persisted translation target from settings", () => {
    const settings = {
      defaults: {},
      connections: [
        {
          id: "conn-1",
          name: "OpenAI",
          provider: "openai" as const,
          baseUrl: "https://api.openai.com/v1",
          models: [
            {
              id: "model-1",
              name: "GPT-4",
              model: "gpt-4o",
              capabilities: ["text-generation" as const],
              builtinAgent: false,
            },
          ],
        },
      ],
      desktopAgents: [],
    };
    const provenance = {
      runId: "run-1",
      targetLanguage: "zh-CN",
      provider: "openai",
      model: "gpt-4o",
    };
    expect(
      persistedTranslationTarget(
        settings as unknown as Parameters<typeof persistedTranslationTarget>[0],
        provenance,
      ),
    ).toEqual({
      type: "model",
      connectionId: "conn-1",
      modelId: "model-1",
    });
  });
});
