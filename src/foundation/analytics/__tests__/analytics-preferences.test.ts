import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, it, vi } from "vitest";

import { useAnalyticsPreferencesStore } from "../analytics-preferences";

const calls = vi.hoisted(() => ({
  load: vi.fn<() => Promise<unknown>>(),
  get: vi.fn<() => Promise<unknown>>(),
  set: vi.fn<(key: string, value: unknown) => Promise<void>>(),
  save: vi.fn<() => Promise<void>>(),
  invoke: vi.fn<(command: string, args?: { installationId: string }) => Promise<unknown>>(),
  stored: undefined as unknown,
}));

vi.mock("@tauri-apps/plugin-store", () => ({ load: calls.load }));
vi.mock("../../ipc/invoke-command", () => ({ invokeCommand: calls.invoke }));

const disk = { get: calls.get, set: calls.set, save: calls.save };
const installationId = "f1f2f34a-9fc2-4d9c-8b6e-29c137bbfbe4";
const utcToday = () => new Date().toISOString().slice(0, 10);

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

it("enables analytics on first launch and persists a new installation identifier", async () => {
  const store = useAnalyticsPreferencesStore();
  await store.load();

  expect(store.enabled).toBe(true);
  expect(calls.stored).toMatchObject({
    enabled: true,
    installationId: expect.stringMatching(
      /^[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i,
    ),
    lastSentDate: null,
  });
  expect(calls.save).toHaveBeenCalledOnce();
});

it("sends the first app-start event automatically and only once per UTC day across launches", async () => {
  const store = useAnalyticsPreferencesStore();
  await store.load();

  await expect(store.recordAppStarted()).resolves.toBe("sent");
  expect(calls.invoke).toHaveBeenCalledOnce();
  expect(calls.stored).toMatchObject({ lastSentDate: utcToday() });

  setActivePinia(createPinia());
  const nextLaunch = useAnalyticsPreferencesStore();
  await nextLaunch.load();

  await expect(nextLaunch.recordAppStarted()).resolves.toBe("disabled");
  expect(calls.invoke).toHaveBeenCalledOnce();
});

it("does not resend an installation already recorded for the current UTC day", async () => {
  calls.stored = {
    enabled: true,
    installationId,
    lastSentDate: utcToday(),
  };

  const store = useAnalyticsPreferencesStore();
  await store.load();

  await expect(store.recordAppStarted()).resolves.toBe("disabled");
  expect(calls.invoke).not.toHaveBeenCalled();
});

it("persists opt-out across launches and creates a fresh identifier when re-enabled", async () => {
  const store = useAnalyticsPreferencesStore();
  await store.load();
  const initialId = (calls.stored as { installationId: string }).installationId;

  await store.setEnabled(false);
  expect(store.enabled).toBe(false);
  expect(calls.stored).toEqual({
    enabled: false,
    installationId: null,
    lastSentDate: null,
  });
  await expect(store.recordAppStarted()).resolves.toBe("disabled");

  setActivePinia(createPinia());
  const nextLaunch = useAnalyticsPreferencesStore();
  await nextLaunch.load();
  expect(nextLaunch.enabled).toBe(false);
  await expect(nextLaunch.recordAppStarted()).resolves.toBe("disabled");
  expect(calls.invoke).not.toHaveBeenCalled();

  await nextLaunch.setEnabled(true);
  expect(nextLaunch.enabled).toBe(true);
  expect((calls.stored as { installationId: string }).installationId).not.toBe(initialId);
});
