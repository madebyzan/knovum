import { describe, expect, it } from "vitest";

import { canStartWith, resolveAiReadiness } from "../ai-readiness";
import { DEFAULT_AI_MODEL_SETTINGS } from "../schema/model-settings";
import { model, settingsWith, sources, writer } from "./support/ai-readiness-fixtures";

describe("routed readiness", () => {
  const requirement = [{ kind: "routed", purpose: "writing" }] as const;

  it("is ready when the routed executor can serve the purpose", () => {
    const settings = settingsWith((value) => {
      writer(value);
      value.defaults.writing = { type: "model", connectionId: "provider", modelId: "writer" };
    });
    expect(resolveAiReadiness(requirement, sources({ settings }))).toEqual({ state: "ready" });
  });

  it("sends the person to routing when a compatible model only needs assigning", () => {
    const settings = settingsWith(writer);
    expect(resolveAiReadiness(requirement, sources({ settings }))).toEqual({
      state: "configure",
      purpose: "writing",
      pane: "aiRouting",
    });
  });

  it("sends the person to providers when no compatible model exists", () => {
    const readiness = resolveAiReadiness(requirement, sources({ settings: DEFAULT_AI_MODEL_SETTINGS }));
    expect(readiness).toMatchObject({ state: "configure", pane: "aiProviders" });
    expect(canStartWith(readiness)).toBe(false);
  });

  it("does not start when the settings cannot be read", () => {
    const readiness = resolveAiReadiness(requirement, sources({ settingsFailed: true }));
    expect(readiness).toEqual({ state: "unavailable", source: "settings" });
    expect(canStartWith(readiness)).toBe(false);
  });

  it("offers a model choice only while a compatible model is configured", () => {
    const choice = [{ kind: "modelChoice", purpose: "writing" }] as const;
    expect(resolveAiReadiness(choice, sources({ settings: settingsWith(writer) }))).toEqual({
      state: "ready",
    });
    expect(resolveAiReadiness(choice, sources({ settings: DEFAULT_AI_MODEL_SETTINGS }))).toMatchObject({
      state: "configure",
      pane: "aiProviders",
    });
  });
});

describe("local readiness", () => {
  it("uses the selected model of a capability and reports what remains to download", () => {
    const models = [
      model({ id: "whisperTiny", selected: false, verified: true }),
      model({ installedBytes: 120 }),
    ];
    const readiness = resolveAiReadiness(
      [{ kind: "localCapability", capability: "transcription" }],
      sources({ models }),
    );
    expect(readiness).toEqual({ state: "download", models: [models[1]], bytes: 380 });
    expect(canStartWith(readiness)).toBe(true);
  });

  it("uses a fixed model that has no selection", () => {
    const embedding = model({
      id: "qwen3Embedding",
      capability: "embedding",
      selected: false,
      fixed: true,
      verified: true,
    });
    expect(
      resolveAiReadiness(
        [{ kind: "localCapability", capability: "embedding" }],
        sources({ models: [embedding] }),
      ),
    ).toEqual({ state: "ready" });
  });

  it("refuses a model this build does not ship", () => {
    const readiness = resolveAiReadiness(
      [{ kind: "localModel", model: "voxCpm2" }],
      sources({ models: [model({})] }),
    );
    expect(readiness).toEqual({ state: "unavailable", source: "platform" });
  });

  it("does not start with an unknown resource state when the catalog cannot be read", () => {
    const readiness = resolveAiReadiness(
      [{ kind: "localModel", model: "voxCpm2" }],
      sources({ modelsFailed: true }),
    );
    expect(readiness).toEqual({ state: "unavailable", source: "models" });
    expect(canStartWith(readiness)).toBe(false);
  });
});

describe("several requirements", () => {
  it("adds up every model still to download", () => {
    const models = [
      model({ id: "whisperSmall", totalBytes: 100 }),
      model({ id: "voxCpm2", name: "VoxCPM2", capability: "speechSynthesis", totalBytes: 50 }),
    ];
    expect(
      resolveAiReadiness(
        [
          { kind: "localModel", model: "whisperSmall" },
          { kind: "localModel", model: "voxCpm2" },
          { kind: "localModel", model: "whisperSmall" },
        ],
        sources({ models }),
      ),
    ).toEqual({ state: "download", models, bytes: 150 });
  });

  it("is decided by whatever stops the feature", () => {
    const readiness = resolveAiReadiness(
      [
        { kind: "localModel", model: "whisperSmall" },
        { kind: "routed", purpose: "writing" },
      ],
      sources({ models: [model({})], settings: DEFAULT_AI_MODEL_SETTINGS }),
    );
    expect(readiness).toMatchObject({ state: "configure", purpose: "writing" });
  });

  it("needs nothing when there is nothing to require", () => {
    expect(resolveAiReadiness([], sources({}))).toEqual({ state: "ready" });
  });
});
