import { beforeEach, expect, it, vi } from "vitest";

import { setDesktopAgentEnabled } from "../desktop-agent-settings";
import { aiModelSettingsSchema, defaultAiModelSettings } from "../schema/model-settings";

const state = vi.hoisted(() => ({
  settings: {} as typeof defaultAiModelSettings,
  flush: vi.fn<() => Promise<void>>(),
}));
vi.mock("../ai-settings-store", () => ({ useAiSettingsStore: () => state }));

beforeEach(() => {
  state.settings = structuredClone(defaultAiModelSettings);
  state.settings.desktopAgents = [
    { kind: "codex", enabled: true },
    { kind: "gemini_cli", enabled: true },
  ];
  state.settings.defaults.defaultTarget = { type: "agent", runtimeKind: "codex" };
  state.settings.defaults.writing = { type: "agent", runtimeKind: "codex" };
  state.settings.defaults.translation = { type: "agent", runtimeKind: "gemini_cli" };
  state.flush.mockReset().mockResolvedValue(undefined);
});

it("disables an Agent and clears exactly the defaults that reference it", async () => {
  await setDesktopAgentEnabled("codex", false);
  expect(state.settings.defaults.defaultTarget).toBeNull();
  expect(state.settings.defaults.writing).toBeNull();
  expect(state.settings.defaults.translation).toEqual({ type: "agent", runtimeKind: "gemini_cli" });
  expect(aiModelSettingsSchema.safeParse(state.settings).success).toBe(true);
});

it("restores the enabled Agent and its routes when saving fails", async () => {
  const previous = structuredClone(state.settings);
  const failure = new Error("save failed");
  state.flush.mockRejectedValueOnce(failure);
  await expect(setDesktopAgentEnabled("codex", false)).rejects.toBe(failure);
  expect(state.settings).toEqual(previous);
});
