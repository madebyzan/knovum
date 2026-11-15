import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, it, vi } from "vitest";

import { useAnalyticsPreferencesStore } from "../analytics-preferences";

const calls = vi.hoisted(() => ({
  load: vi.fn(),
  get: vi.fn(),
  set: vi.fn(),
  save: vi.fn(),
  invoke: vi.fn(),
  stored: undefined as unknown,
}));

vi.mock("@tauri-apps/plugin-store", () => ({ load: calls.load }));
vi.mock("../../ipc/invoke-command", () => ({ invokeCommand: calls.invoke }));

const disk = { get: calls.get, set: calls.set, save: calls.save };
const installationId = "f1f2f34a-9fc2-4d9c-8b6e-29c137bbfbe4";

beforeEach(() => {
  setActivePinia(createPinia());
  vi.resetAllMocks();
  calls.stored = undefined;
  calls.load.mockResolvedValue(disk);
  calls.get.mockImplementation(async () => calls.stored);
  calls.set.mockImplementation(async (_key, value) => {
    calls.stored = value;
  });
  calls.save.mockResolvedValue(undefined);
  calls.invoke.mockResolvedValue("sent");
});

it("enables fresh installations and records one daily app-start event", async () => {
  const store = useAnalyticsPreferencesStore();
  await store.load();

  expect(store.enabled).toBe(true);
  expect(calls.stored).toMatchObject({
    schemaVersion: 1,
    enabled: true,
    installationId: expect.stringMatching(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i,
    ),
    lastSentDate: null,
  });

  await expect(store.recordAppStarted()).resolves.toBe("sent");
  expect(calls.invoke).toHaveBeenCalledOnce();
  expect(calls.stored).toMatchObject({ lastSentDate: new Date().toISOString().slice(0, 10) });
  await store.recordAppStarted();
  expect(calls.invoke).toHaveBeenCalledOnce();
});

it("preserves an explicit opt-out saved by the previous preference format", async () => {
  calls.stored = {
    schemaVersion: 1,
    enabled: false,
    installationId: null,
    lastSentDate: null,
  };

  const store = useAnalyticsPreferencesStore();
  await store.load();

  expect(store.enabled).toBe(false);
  await expect(store.recordAppStarted()).resolves.toBe("disabled");
  expect(calls.invoke).not.toHaveBeenCalled();
  expect(calls.set).not.toHaveBeenCalled();
});

it("does not resend an installation already recorded for the current UTC day", async () => {
  calls.stored = {
    schemaVersion: 1,
    enabled: true,
    installationId,
    lastSentDate: new Date().toISOString().slice(0, 10),
  };

  const store = useAnalyticsPreferencesStore();
  await store.load();

  await expect(store.recordAppStarted()).resolves.toBe("disabled");
  expect(calls.invoke).not.toHaveBeenCalled();
});

it("clears the installation identifier on opt-out and creates a new one on opt-in", async () => {
  const store = useAnalyticsPreferencesStore();
  await store.load();
  const initialId = (calls.stored as { installationId: string }).installationId;

  await store.setEnabled(false);
  expect(store.enabled).toBe(false);
  expect(calls.stored).toMatchObject({ enabled: false, installationId: null, lastSentDate: null });

  await store.setEnabled(true);
  expect(store.enabled).toBe(true);
  expect(calls.stored).toMatchObject({ enabled: true });
  expect((calls.stored as { installationId: string }).installationId).not.toBe(initialId);
});
