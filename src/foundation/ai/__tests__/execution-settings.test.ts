import { beforeEach, expect, it, vi } from "vitest";

import { setDefaultExecutionTarget, setDesktopAgentEnabled } from "../execution-settings";
import { aiModelSettingsSchema, DEFAULT_AI_MODEL_SETTINGS } from "../schema/model-settings";

const state = vi.hoisted(() => ({
  settings: {} as typeof DEFAULT_AI_MODEL_SETTINGS,
  flush: vi.fn<() => Promise<void>>(),
}));
vi.mock("../ai-settings-store", () => ({ useAiSettingsStore: () => state }));

beforeEach(() => {
  state.settings = structuredClone(DEFAULT_AI_MODEL_SETTINGS);
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

it("queues route edits until a failed Agent mutation has rolled back", async () => {
  let rejectSave!: (error: Error) => void;
  state.flush.mockReturnValueOnce(
    new Promise<void>((_resolve, reject) => {
      rejectSave = reject;
    }),
  );
  const disabling = setDesktopAgentEnabled("codex", false);
  const failure = new Error("Agent save failed");
  const settlement = disabling.catch((error: unknown) => error);
  await vi.waitFor(() => expect(state.flush).toHaveBeenCalledOnce());
  const target = { type: "agent", runtimeKind: "codex" } as const;
  const route = setDefaultExecutionTarget("translation", target);
  expect(state.settings.defaults.translation).toEqual({ type: "agent", runtimeKind: "gemini_cli" });
  rejectSave(failure);
  expect(await settlement).toBe(failure);
  await route;
  expect(state.settings.defaults.translation).toEqual(target);
});

it("restores only the changed route after persistence fails", async () => {
  const previous = structuredClone(state.settings.defaults);
  const failure = new Error("Route save failed");
  state.flush.mockRejectedValueOnce(failure);
  await expect(setDefaultExecutionTarget("writing", null)).rejects.toBe(failure);
  expect(state.settings.defaults).toEqual(previous);
});
