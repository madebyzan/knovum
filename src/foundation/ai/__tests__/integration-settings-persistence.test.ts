import { createPinia, setActivePinia } from "pinia";
import { afterEach, beforeEach, expect, it, vi } from "vitest";

import { useAiSettingsStore } from "../ai-settings-store";
import {
  loadCreationLibrary,
  resetCreationLibrary,
  saveAiModelSettings,
  saveCreationLibrary,
} from "../integration-settings";
import { creationLibrarySchema, defaultCreationLibrary } from "../schema/creation-library";
import { aiModelSettingsSchema, defaultAiModelSettings } from "../schema/model-settings";

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
vi.mock("@tauri-apps/plugin-log", () => ({
  error: vi.fn<(message: string) => Promise<void>>().mockResolvedValue(undefined),
}));

beforeEach(() => {
  vi.useFakeTimers();
  vi.clearAllMocks();
  setActivePinia(createPinia());
});
afterEach(() => {
  vi.restoreAllMocks();
  vi.useRealTimers();
});

it("canonicalizes each save once before persisting", async () => {
  const aiParse = vi.spyOn(aiModelSettingsSchema, "parse");
  const creationParse = vi.spyOn(creationLibrarySchema, "parse");

  await saveAiModelSettings(structuredClone(defaultAiModelSettings));
  await saveCreationLibrary(structuredClone(defaultCreationLibrary));

  expect(aiParse).toHaveBeenCalledOnce();
  expect(creationParse).toHaveBeenCalledOnce();
  expect(boundary.set).toHaveBeenCalledTimes(2);
});

it("continues queued writes and reads after a store write fails", async () => {
  const cause = new Error("Store unavailable");
  boundary.save.mockRejectedValueOnce(cause);
  boundary.get.mockResolvedValueOnce({ schemaVersion: 1, value: defaultCreationLibrary });

  const failedWrite = saveCreationLibrary(structuredClone(defaultCreationLibrary));
  const restoredWrite = resetCreationLibrary();
  const read = loadCreationLibrary();

  await expect(failedWrite).rejects.toBe(cause);
  await expect(restoredWrite).resolves.toBeUndefined();
  await expect(read).resolves.toEqual(defaultCreationLibrary);
  expect(boundary.save).toHaveBeenCalledTimes(2);
  expect(boundary.save.mock.invocationCallOrder[1]).toBeLessThan(boundary.get.mock.invocationCallOrder[0]!);
});

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

it("rejects URL credentials before validation or store writes", async () => {
  const value = structuredClone(defaultAiModelSettings);
  value.connections.push({
    id: "unsafe",
    name: "Unsafe",
    provider: "openai",
    baseUrl: "https://user:secret@models.example.test/v1",
    models: [],
  });

  await expect(saveAiModelSettings(value)).rejects.toThrow("Invalid AI provider URL");

  expect(boundary.invoke).not.toHaveBeenCalled();
  expect(boundary.set).not.toHaveBeenCalled();
  expect(boundary.save).not.toHaveBeenCalled();
});
