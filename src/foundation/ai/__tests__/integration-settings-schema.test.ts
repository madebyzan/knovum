import { describe, expect, it } from "vitest";

import {
  decodeAiModelSettings,
  decodeCreationLibrary,
  defaultAiModelSettings,
  defaultCreationLibrary,
} from "../integration-settings";

describe("integration settings persisted contracts", () => {
  it("defaults without built-in AI models or connections", () => {
    expect(defaultAiModelSettings.connections).toEqual([]);
    expect(defaultAiModelSettings.defaults).toEqual({
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

  it("accepts an enabled Agent as every current execution route", () => {
    const value = structuredClone(defaultAiModelSettings);
    value.desktopAgents.push({ kind: "codex", enabled: true });
    const target = { type: "agent" as const, kind: "codex" as const };
    value.defaults = {
      writing: target,
      knowledge: target,
      visualUnderstanding: target,
      translation: target,
      imageGeneration: target,
      videoGeneration: target,
    };
    expect(decodeAiModelSettings({ schemaVersion: 1, value })).toEqual(value);
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

  it("persists multi-model Creation Workflows against one exact Workflow version", () => {
    const value = structuredClone(defaultCreationLibrary);
    value.creationWorkflows.push({
      id: "social",
      name: "Product launch",
      workflowId: "social-content-kit",
      workflowVersion: 1,
      targets: {
        text: {
          actionId: "write-social-copy",
          target: { type: "model", connectionId: "custom-qwen", modelId: "qwen-plus" },
        },
        image: {
          actionId: "create-companion-image",
          target: { type: "model", connectionId: "custom-qwen", modelId: "qwen-image" },
        },
      },
      parameters: { platform: "xiaohongshu", aspectRatio: "9:16" },
    });

    expect(decodeCreationLibrary({ schemaVersion: 1, value })).toEqual(value);
  });

  it("rejects unversioned or non-current envelopes", () => {
    expect(() => decodeAiModelSettings({ schemaVersion: 2, value: defaultAiModelSettings })).toThrow(/./u);
  });
});
