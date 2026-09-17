import { describe, expect, it } from "vitest";
import { isProxy, reactive } from "vue";

import type { AgentDescriptor, AgentInstallation } from "../agent-models";
import { availableExecutionTargets, defaultExecutionTarget } from "../ai-execution";
import { defaultAiModelSettings } from "../integration-settings";

const descriptor: AgentDescriptor = {
  kind: "codex",
  displayName: "OpenAI Codex",
  executableNames: ["codex"],
  help: {
    connection: "node_adapter",
    documentationUrl: "https://github.com/agentclientprotocol/codex-acp",
    setupUrl: "https://github.com/agentclientprotocol/registry/blob/main/codex-acp/agent.json",
  },
};

function installation(): AgentInstallation {
  return {
    kind: "codex",
    detected: true,
    executablePath: "/usr/local/bin/codex",
  };
}

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
    "offers an installed Agent for %s without asserting protocol readiness",
    (purpose) => {
      const settings = structuredClone(defaultAiModelSettings);
      settings.desktopAgents.push({ kind: "codex", enabled: true });
      const [option] = availableExecutionTargets(settings, purpose, [descriptor], [installation()]);
      expect(option).toMatchObject({
        target: { type: "agent", kind: "codex" },
        executorType: "agent",
        disabled: false,
      });
    },
  );

  it("does not treat installation as verified execution capability", () => {
    const settings = structuredClone(defaultAiModelSettings);
    settings.desktopAgents.push({ kind: "codex", enabled: true });
    const [option] = availableExecutionTargets(settings, "image-generation", [descriptor], [installation()]);
    expect(option).toMatchObject({
      disabled: false,
      availability: "unverified",
    });
  });

  it("keeps an enabled Agent selectable when it is not found in the inspected PATH", () => {
    const settings = structuredClone(defaultAiModelSettings);
    settings.desktopAgents.push({ kind: "codex", enabled: true });
    const missing = { ...installation(), detected: false, executablePath: null };
    const [option] = availableExecutionTargets(settings, "visual-understanding", [descriptor], [missing]);
    expect(option).toMatchObject({
      disabled: false,
      availability: "unavailable",
    });
  });
});
