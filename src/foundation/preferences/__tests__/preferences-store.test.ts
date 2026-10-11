import { createPinia, setActivePinia } from "pinia";
import { beforeEach, expect, it, vi } from "vitest";

import { runApplicationTransition } from "../../lifecycle/application-transition";
import { usePreferencesStore } from "../preferences-store";

const calls = vi.hoisted(() => ({
  load: vi.fn<() => Promise<unknown>>(),
  set: vi.fn<(key: string, value: unknown) => Promise<void>>(),
  save: vi.fn<() => Promise<void>>(),
  stored: undefined as unknown,
}));
vi.mock("@tauri-apps/plugin-store", () => ({ load: calls.load }));
vi.mock("@tauri-apps/plugin-os", () => ({ locale: async () => "en" }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    theme: async () => "light",
    setTheme: async () => {},
    onThemeChanged: async () => () => {},
  }),
}));
vi.mock("../../ipc/invoke-command", () => ({ invokeCommand: async () => {} }));
vi.mock("../../i18n/instance", () => ({
  i18n: { global: { locale: { value: "en" } } },
  systemLocale: () => "en",
}));
const disk = { get: async () => undefined, set: calls.set, save: calls.save };

beforeEach(() => {
  setActivePinia(createPinia());
  vi.resetAllMocks();
  calls.stored = undefined;
  calls.load.mockResolvedValue(disk);
  calls.set.mockImplementation(async (_key, value) => {
    calls.stored = value;
  });
  calls.save.mockResolvedValue(undefined);
});

it("serializes a newer preference edit behind an older in-flight persistence operation", async () => {
  const store = usePreferencesStore();
  await store.load();
  let release!: () => void;
  calls.load.mockReturnValueOnce(
    new Promise((resolve) => {
      release = () => resolve(disk);
    }),
  );
  const older = store.setDensity("comfortable");
  const newer = store.setTextSize("large");
  await vi.waitFor(() => expect(release).toBeDefined());
  release();
  await Promise.all([older, newer]);
  expect(calls.stored).toMatchObject({ value: { density: "comfortable", textSize: "large" } });
});

it("rejects new preference mutations during exit", async () => {
  const store = usePreferencesStore();
  await store.load();
  await runApplicationTransition(async () => {
    await expect(store.setDensity("comfortable")).rejects.toThrow("transitioning");
  });
  expect(calls.set).not.toHaveBeenCalled();
});

it("waits for accepted preference writes without rewriting a clean snapshot", async () => {
  const store = usePreferencesStore();
  await store.load();
  await store.flush();
  expect(calls.save).not.toHaveBeenCalled();
  let release!: () => void;
  calls.save.mockReturnValueOnce(
    new Promise<void>((resolve) => {
      release = resolve;
    }),
  );
  const saving = store.setTextSize("large");
  await vi.waitFor(() => expect(release).toBeDefined());
  const finished = vi.fn<() => void>();
  const closing = store.flush().then(finished);
  await Promise.resolve();
  expect(finished).not.toHaveBeenCalled();
  release();
  await Promise.all([saving, closing]);
  expect(calls.save).toHaveBeenCalledOnce();
});

it("rejects exit while dirty preferences still cannot be persisted", async () => {
  const store = usePreferencesStore();
  await store.load();
  const failure = new Error("Disk unavailable");
  calls.save.mockRejectedValue(failure);
  await expect(store.setTextSize("large")).rejects.toBe(failure);
  await expect(store.flush()).rejects.toBe(failure);
});

it("does not overwrite unloaded preferences", async () => {
  await usePreferencesStore().flush();
  expect(calls.set).not.toHaveBeenCalled();
});

it("loads absent optional preferences without rewriting disk and saves a complete value on edit", async () => {
  calls.load.mockResolvedValue({
    ...disk,
    get: async () => ({
      schemaVersion: 1,
      value: { theme: "dark", density: "comfortable", textSize: "large", locale: "zh-CN" },
    }),
  });
  const store = usePreferencesStore();
  await store.load();
  await store.flush();
  expect(store.theme).toBe("dark");
  expect(store.locale).toBe("zh-CN");
  expect(calls.set).not.toHaveBeenCalled();
  expect(calls.save).not.toHaveBeenCalled();

  await store.setDensity("compact");
  expect(calls.stored).toEqual({
    schemaVersion: 1,
    value: {
      theme: "dark",
      density: "compact",
      textSize: "large",
      locale: "zh-CN",
      backgroundTaskNotifications: true,
      workbench: {
        sidebarVisible: true,
        sidebarWidth: 224,
        rightSidebarVisible: false,
        rightSidebarWidth: 380,
      },
    },
  });
});
