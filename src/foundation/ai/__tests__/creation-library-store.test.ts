import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, it, vi } from "vitest";

import { runApplicationTransition } from "../../lifecycle/application-transition";
import { useCreationLibraryStore } from "../creation-library-store";
import type { GenerationPreset } from "../schema/models";

const calls = vi.hoisted(() => ({
  save: vi.fn<(library: unknown) => Promise<void>>(),
  reset: vi.fn<() => Promise<void>>(),
}));
vi.mock("../integration-settings", () => ({
  loadCreationLibrary: async () => ({ generationPresets: [] }),
  saveCreationLibrary: calls.save,
  resetCreationLibrary: calls.reset,
}));
const preset: GenerationPreset = {
  id: "preset",
  name: "Original",
  actionId: "image-generation",
  target: { type: "agent", runtimeKind: "codex" },
  parameters: {},
};

beforeEach(() => {
  setActivePinia(createPinia());
  vi.resetAllMocks();
  calls.save.mockResolvedValue(undefined);
  calls.reset.mockResolvedValue(undefined);
});

it("rejects new preset edits and resets during exit", async () => {
  const store = useCreationLibraryStore();
  await store.ensureLoaded();
  await runApplicationTransition(async () => {
    await expect(store.reset()).rejects.toThrow("transitioning");
    await expect(store.addPreset(preset)).rejects.toThrow("transitioning");
  });
  expect(store.library.generationPresets).toEqual([]);
  expect(calls.reset).not.toHaveBeenCalled();
});

it("drains admitted preset edits before close", async () => {
  const store = useCreationLibraryStore();
  await store.ensureLoaded();
  let release!: () => void;
  calls.save.mockReturnValueOnce(
    new Promise<void>((resolve) => {
      release = resolve;
    }),
  );
  const adding = store.addPreset(preset);
  await vi.waitFor(() => expect(release).toBeDefined());
  const renaming = store.renamePreset(preset.id, "Latest");
  const settled = vi.fn<() => void>();
  const closing = store.flush().then(settled);
  await Promise.resolve();
  expect(settled).not.toHaveBeenCalled();
  release();
  await Promise.all([adding, renaming, closing]);
  expect(calls.save).toHaveBeenLastCalledWith({ generationPresets: [{ ...preset, name: "Latest" }] });
});

it("retains an unsaved preset and propagates its persistence failure", async () => {
  const store = useCreationLibraryStore();
  await store.ensureLoaded();
  const failure = new Error("Save failed");
  calls.save.mockRejectedValue(failure);
  await expect(store.addPreset(preset)).rejects.toBe(failure);
  await expect(store.flush()).rejects.toBe(failure);
  expect(store.library.generationPresets).toEqual([preset]);
});

it("duplicates independently and updates and removes presets by identity", async () => {
  const store = useCreationLibraryStore();
  await store.ensureLoaded();
  await store.addPreset(preset);
  await store.duplicatePreset(preset.id, "Copy");
  const copy = store.library.generationPresets[1]!;
  expect(copy.id).not.toBe(preset.id);
  expect(copy.target).not.toBe(store.library.generationPresets[0]!.target);
  await store.updatePreset({ ...copy, parameters: { seed: 42 } });
  await store.removePreset(preset.id);
  expect(store.library.generationPresets).toEqual([{ ...copy, parameters: { seed: 42 } }]);
});
