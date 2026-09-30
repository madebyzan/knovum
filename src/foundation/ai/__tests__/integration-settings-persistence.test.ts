import { createPinia, setActivePinia } from "pinia";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { useAiSettingsStore } from "../ai-settings-store";

const boundary = vi.hoisted(() => ({
  get: vi.fn<(key: string) => Promise<unknown>>(),
  set: vi.fn<(key: string, value: unknown) => Promise<void>>().mockResolvedValue(undefined),
  save: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
  invoke: vi
    .fn<(command: string, payload: Record<string, unknown>) => Promise<void>>()
    .mockResolvedValue(undefined),
}));

vi.mock("@tauri-apps/plugin-store", () => ({
  LazyStore: class {
    get = boundary.get;
    set = boundary.set;
    save = boundary.save;
  },
}));
vi.mock("../../ipc/invoke-command", () => ({ invokeCommand: boundary.invoke }));

beforeEach(() => {
  vi.useFakeTimers();
  vi.clearAllMocks();
  setActivePinia(createPinia());
});
afterEach(() => vi.useRealTimers());

it("loads optional routing preferences without a reset and saves through the existing validated boundary", async () => {
  const stored = {
    schemaVersion: 1,
    value: {
      connections: [],
      desktopAgents: [{ kind: "codex", enabled: true }],
      defaults: { visualUnderstanding: { type: "agent", runtimeKind: "codex" } },
    },
  };
  boundary.get.mockResolvedValue(structuredClone(stored));
  const store = useAiSettingsStore();

  await store.ensureLoaded();
  await vi.advanceTimersByTimeAsync(1200);

  expect(boundary.get).toHaveBeenCalledWith("ai.models");
  expect(store.ready).toBe(true);
  expect(store.loadError).toBeUndefined();
  expect(store.settings.desktopAgents).toEqual(stored.value.desktopAgents);
  expect(store.settings.defaults.visualUnderstanding).toEqual(stored.value.defaults.visualUnderstanding);
  expect(store.settings.defaults.defaultTarget).toBeNull();
  expect(boundary.set).not.toHaveBeenCalled();
  expect(boundary.save).not.toHaveBeenCalled();
  expect(boundary.invoke).not.toHaveBeenCalled();

  store.settings.defaults.defaultTarget = { type: "agent", runtimeKind: "codex" };
  await store.flush();
  await vi.advanceTimersByTimeAsync(1200);

  const canonical = {
    connections: [],
    desktopAgents: stored.value.desktopAgents,
    defaults: {
      defaultTarget: { type: "agent", runtimeKind: "codex" },
      writing: null,
      knowledge: null,
      visualUnderstanding: stored.value.defaults.visualUnderstanding,
      translation: null,
      imageGeneration: null,
      videoGeneration: null,
    },
  };
  expect(boundary.invoke).toHaveBeenCalledExactlyOnceWith("plugin:ai|validate_model_settings", {
    settings: canonical,
  });
  expect(boundary.set).toHaveBeenCalledExactlyOnceWith("ai.models", { schemaVersion: 1, value: canonical });
  expect(boundary.save).toHaveBeenCalledOnce();
});
