import { describe, expect, it } from "vitest";
import { isProxy, reactive } from "vue";

import { availableExecutionTargets, defaultExecutionTarget } from "../ai-execution";
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
        executorType: "model",
        disabled: false,
      });
    },
  );

  it("does not mix desktop Agents into model execution routes", () => {
    const settings = structuredClone(defaultAiModelSettings);
    settings.desktopAgents.push({ kind: "codex", enabled: true });
    expect(availableExecutionTargets(settings, "image-generation")).toEqual([]);
  });
});
