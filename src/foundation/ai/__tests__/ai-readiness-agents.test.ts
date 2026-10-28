import { describe, expect, it } from "vitest";

import { aiSourcesOf, canStartWith, desktopAgentPresence, resolveAiReadiness } from "../ai-readiness";
import { DEFAULT_AI_MODEL_SETTINGS } from "../schema/model-settings";
import { settingsWith, sources } from "./support/ai-readiness-fixtures";

/** Codex as detection reports it: the app may be present while its command line is not. */
function presence(executablePath: string | null) {
  return desktopAgentPresence(
    [{ kind: "codex", displayName: "Codex", executableNames: ["codex"], help: {} as never }],
    [{ kind: "codex", detected: true, executablePath }],
  );
}

describe("desktop Agent readiness", () => {
  const codex = { type: "agent", runtimeKind: "codex" } as const;
  const requirement = [{ kind: "routed", purpose: "knowledge", target: codex }] as const;
  const enabled = settingsWith((value) => value.desktopAgents.push({ kind: "codex", enabled: true }));

  it("is ready when the chosen Agent is enabled and its command line is installed", () => {
    expect(
      resolveAiReadiness(requirement, sources({ settings: enabled, agents: presence("/bin/codex") })),
    ).toEqual({
      state: "ready",
    });
  });

  it("asks for an installation when only the desktop app was found", () => {
    const readiness = resolveAiReadiness(requirement, sources({ settings: enabled, agents: presence(null) }));
    expect(readiness).toEqual({
      state: "install",
      agent: { kind: "codex", name: "Codex", installed: false },
    });
    expect(canStartWith(readiness)).toBe(false);
  });

  it("asks for configuration when the chosen Agent is not enabled", () => {
    const readiness = resolveAiReadiness(
      requirement,
      sources({ settings: DEFAULT_AI_MODEL_SETTINGS, agents: presence("/bin/codex") }),
    );
    expect(readiness).toMatchObject({ state: "configure", purpose: "knowledge" });
  });

  it("waits for detection, and does not start when it fails", () => {
    expect(resolveAiReadiness(requirement, sources({ settings: enabled }))).toEqual({ state: "checking" });
    expect(resolveAiReadiness(requirement, sources({ settings: enabled, agentsFailed: true }))).toEqual({
      state: "unavailable",
      source: "agents",
    });
  });
});

describe("desktop Agent detection", () => {
  it("is read only for a route that resolves to a desktop Agent", () => {
    const enabled = settingsWith((value) => value.desktopAgents.push({ kind: "codex", enabled: true }));
    const conversation = [
      { kind: "routed", purpose: "knowledge", target: { type: "agent", runtimeKind: "codex" } },
    ] as const;
    expect([...aiSourcesOf(conversation, undefined)]).toEqual(["settings"]);
    expect([...aiSourcesOf(conversation, enabled)]).toEqual(["settings", "agents"]);
    expect([...aiSourcesOf([{ kind: "routed", purpose: "writing" }], enabled)]).toEqual(["settings"]);
  });
});
