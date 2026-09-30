import { describe, expect, it } from "vitest";
import { isProxy, reactive } from "vue";

import { availableExecutionTargets, availableModelTargets, defaultExecutionTarget } from "../ai-execution";
import { encodeExecutionTarget, decodeExecutionTarget } from "../execution-target-codec";
import { defaultAiModelSettings } from "../integration-settings";

describe("execution target options", () => {
  it("returns a plain default target when settings are reactive", () => {
    const settings = reactive(structuredClone(defaultAiModelSettings));
    settings.defaults.knowledge = {
      type: "model",
      connectionId: "local-provider",
      modelId: "knowledge-model",
    };

    const target = defaultExecutionTarget(settings, "knowledge");

    expect(target).toEqual({
      type: "model",
      connectionId: "local-provider",
      modelId: "knowledge-model",
    });
    expect(isProxy(target)).toBe(false);
    expect(() => structuredClone(target)).not.toThrow();
  });

  it.each(["visual-understanding", "image-generation", "video-generation"] as const)(
    "offers a configured model with the capability required by %s",
    (purpose) => {
      const settings = structuredClone(defaultAiModelSettings);
      settings.connections.push({
        id: "media",
        name: "Media",
        provider: "openai",
        baseUrl: "https://api.openai.com/v1/",
        models: [
          {
            id: "omni",
            name: "Omni",
            model: "omni",
            capabilities: ["vision-understanding", "image-generation", "video-generation"],
          },
        ],
      });
      const [option] = availableExecutionTargets(settings, purpose);
      expect(option).toMatchObject({
        target: { type: "model", connectionId: "media", modelId: "omni" },
      });
    },
  );

  it("does not mix desktop Agents into model execution routes", () => {
    const settings = structuredClone(defaultAiModelSettings);
    settings.desktopAgents.push({ kind: "codex", enabled: true });
    expect(availableModelTargets(settings, "image-generation")).toEqual([]);
  });
});

it("round-trips execution identities containing separator characters", () => {
  const target = { type: "model" as const, connectionId: "provider:v1", modelId: "model:chat" };
  expect(decodeExecutionTarget(encodeExecutionTarget(target))).toEqual(target);
});

it("keeps distinct model references separate when identifiers contain separators", () => {
  const left = { type: "model" as const, connectionId: "a:b", modelId: "c" };
  const right = { type: "model" as const, connectionId: "a", modelId: "b:c" };
  expect(encodeExecutionTarget(left)).not.toBe(encodeExecutionTarget(right));
});

it("inherits an explicitly selected desktop Agent across product tasks without model connections", () => {
  const settings = structuredClone(defaultAiModelSettings);
  settings.desktopAgents = [{ kind: "codex", enabled: true }];
  settings.defaults.defaultTarget = { type: "agent", runtimeKind: "codex" };
  for (const purpose of [
    "writing",
    "knowledge",
    "visual-understanding",
    "translation",
    "image-generation",
    "video-generation",
  ] as const) {
    expect(defaultExecutionTarget(settings, purpose)).toEqual(settings.defaults.defaultTarget);
    expect(availableExecutionTargets(settings, purpose)).toMatchObject([
      { target: settings.defaults.defaultTarget },
    ]);
  }
  settings.defaults.knowledge = { type: "agent", runtimeKind: "qoder_cli" };
  expect(defaultExecutionTarget(settings, "knowledge")).toEqual(settings.defaults.knowledge);
  expect(settings.connections).toEqual([]);
});
it("does not choose an executor just because it was enabled", () => {
  const settings = structuredClone(defaultAiModelSettings);
  settings.desktopAgents = [{ kind: "codex", enabled: true }];
  expect(defaultExecutionTarget(settings, "knowledge")).toBeNull();
});
