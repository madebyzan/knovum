import { describe, expect, it } from "vitest";

import { decodeAiModelSettings } from "../integration-settings";
import { defaultAiModelSettings } from "../schema/model-settings";

describe("integration settings persisted contracts", () => {
  it("defaults without built-in AI models or connections", () => {
    expect(defaultAiModelSettings.connections).toEqual([]);
    expect(defaultAiModelSettings.defaults).toEqual({
      defaultTarget: null,
      writing: null,
      knowledge: null,
      visualUnderstanding: null,
      translation: null,
      imageGeneration: null,
      videoGeneration: null,
    });
  });

  it("accepts the exact current v1 envelopes", () => {
    expect(decodeAiModelSettings({ schemaVersion: 1, value: defaultAiModelSettings })).toEqual(
      defaultAiModelSettings,
    );
  });

  it("accepts user-configured connections without persisting a credential", () => {
    const value = structuredClone(defaultAiModelSettings);
    value.connections.push({
      id: "custom-qwen",
      name: "My Qwen",
      provider: "qwen",
      baseUrl: "https://dashscope.aliyuncs.com/compatible-mode/v1/",
      models: [
        {
          id: "qwen-plus",
          name: "Qwen Plus",
          model: "qwen-plus",
          capabilities: ["text-generation"],
        },
      ],
    });
    value.defaults.writing = { type: "model", connectionId: "custom-qwen", modelId: "qwen-plus" };
    value.defaults.knowledge = { type: "model", connectionId: "custom-qwen", modelId: "qwen-plus" };
    expect(decodeAiModelSettings({ schemaVersion: 1, value })).toEqual(value);
    expect(value.connections[0]).not.toHaveProperty("apiKey");
  });

  it("accepts a provider connection before models are added", () => {
    const value = structuredClone(defaultAiModelSettings);
    value.connections.push({
      id: "openai",
      name: "OpenAI",
      provider: "openai",
      baseUrl: "https://api.openai.com/v1/",
      models: [],
    });
    expect(decodeAiModelSettings({ schemaVersion: 1, value })).toEqual(value);
  });

  it("rejects malformed or unenabled desktop Agent identities", () => {
    const value = structuredClone(defaultAiModelSettings) as unknown as Record<string, unknown>;
    value.defaults = {
      writing: { type: "agent", kind: "codex" },
      knowledge: null,
      visualUnderstanding: null,
      translation: null,
      imageGeneration: null,
      videoGeneration: null,
    };
    expect(() => decodeAiModelSettings({ schemaVersion: 1, value })).toThrow(/./u);
  });

  it("leaves provider capability authority to Rust validation", () => {
    const value = structuredClone(defaultAiModelSettings);
    value.connections.push({
      id: "openai",
      name: "OpenAI",
      provider: "openai",
      baseUrl: "https://api.openai.com/v1/",
      models: [
        {
          id: "video",
          name: "Video",
          model: "gpt-example",
          capabilities: ["video-generation"],
        },
      ],
    });
    expect(decodeAiModelSettings({ schemaVersion: 1, value })).toEqual(value);
  });

  it("rejects incomplete providers instead of persisting editor drafts", () => {
    const value = structuredClone(defaultAiModelSettings);
    value.connections.push({
      id: "incomplete",
      name: "Incomplete",
      provider: "openai",
      baseUrl: "not-a-url",
      models: [],
    });
    expect(() => decodeAiModelSettings({ schemaVersion: 1, value })).toThrow(/./u);
  });

  it.each(["ftp://models.example.test/v1", "https://user:secret@models.example.test/v1", "https://"])(
    "rejects an unsafe provider URL without reflecting credentials: %s",
    (baseUrl) => {
      const value = structuredClone(defaultAiModelSettings);
      value.connections.push({
        id: "unsafe",
        name: "Unsafe",
        provider: "openai",
        baseUrl,
        models: [],
      });
      const decodeInvalidSettings = decodeAiModelSettings.bind(undefined, { schemaVersion: 1, value });
      expect(decodeInvalidSettings).toThrow("Invalid AI provider URL");
      const error = (() => {
        try {
          decodeInvalidSettings();
        } catch (cause) {
          return cause;
        }
        throw new Error("Expected URL validation to fail");
      })();
      expect(String(error)).not.toContain("secret");
    },
  );

  it("rejects unversioned or non-current envelopes", () => {
    expect(() => decodeAiModelSettings({ schemaVersion: 2, value: defaultAiModelSettings })).toThrow(/./u);
  });
});

it("accepts enabled Agent-only product routing", () => {
  const value = structuredClone(defaultAiModelSettings);
  value.desktopAgents = [{ kind: "codex", enabled: true }];
  value.defaults.defaultTarget = { type: "agent", runtimeKind: "codex" };
  value.defaults.visualUnderstanding = { type: "agent", runtimeKind: "codex" };
  expect(decodeAiModelSettings({ schemaVersion: 1, value })).toEqual(value);
});
