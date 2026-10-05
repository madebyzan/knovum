import { createPinia } from "pinia";
import { beforeEach, expect, it, vi } from "vitest";

import { installSettingsPersistence } from "../settings-persistence";

const calls = vi.hoisted(() => ({
  register: vi.fn<(commit: () => Promise<void>) => () => void>(),
  ai: vi.fn<() => Promise<void>>(),
  creation: vi.fn<() => Promise<void>>(),
  preferences: vi.fn<() => Promise<void>>(),
}));
vi.mock("../../foundation/lifecycle/window-close-coordinator", () => ({
  registerWindowCloseCommit: calls.register,
}));
vi.mock("../../foundation/ai/settings-mutations", () => ({ flushAiSettingsMutations: calls.ai }));
vi.mock("../../foundation/ai/creation-library-store", () => ({
  useCreationLibraryStore: () => ({ flush: calls.creation }),
}));
vi.mock("../../foundation/preferences/preferences-store", () => ({
  usePreferencesStore: () => ({ flush: calls.preferences }),
}));

beforeEach(() => {
  vi.resetAllMocks();
  calls.ai.mockResolvedValue(undefined);
  calls.creation.mockResolvedValue(undefined);
  calls.preferences.mockResolvedValue(undefined);
});

it("registers one save barrier per application and waits for all settings owners", async () => {
  const pinia = createPinia();
  installSettingsPersistence(pinia);
  installSettingsPersistence(pinia);
  expect(calls.register).toHaveBeenCalledOnce();
  let release!: () => void;
  calls.ai.mockImplementationOnce(
    () =>
      new Promise<void>((resolve) => {
        release = resolve;
      }),
  );
  const finished = vi.fn<() => void>();
  const closing = calls.register.mock.calls[0]![0]().then(finished);
  await Promise.resolve();
  expect(calls.creation).toHaveBeenCalledOnce();
  expect(calls.preferences).toHaveBeenCalledOnce();
  expect(finished).not.toHaveBeenCalled();
  release();
  await closing;
});

it("preserves every save failure so exit cannot be approved", async () => {
  const errors = [
    new Error("AI save failed"),
    new Error("creation save failed"),
    new Error("preferences save failed"),
  ];
  calls.ai.mockRejectedValueOnce(errors[0]);
  calls.creation.mockRejectedValueOnce(errors[1]);
  calls.preferences.mockRejectedValueOnce(errors[2]);
  installSettingsPersistence(createPinia());
  await expect(calls.register.mock.calls[0]![0]()).rejects.toMatchObject({ errors });
});
