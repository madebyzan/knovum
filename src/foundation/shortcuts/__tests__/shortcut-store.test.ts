import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { DictationShortcutStatus } from "../../../shared/contracts/generated/DictationShortcutStatus";
import { useShortcutsStore } from "../shortcut-store";

const mockStore = vi.hoisted(() => ({
  get: vi.fn<(key: string) => Promise<unknown>>(),
  set: vi.fn<(key: string, val: unknown) => Promise<void>>(),
  save: vi.fn<() => Promise<void>>(),
  stored: undefined as unknown,
}));

const mockInvoke = vi.hoisted(() => vi.fn<(cmd: string, args?: unknown) => Promise<unknown>>());

vi.mock("@tauri-apps/plugin-store", () => ({
  LazyStore: class {
    get = mockStore.get;
    set = mockStore.set;
    save = mockStore.save;
  },
}));

vi.mock("../../ipc/invoke-command", () => ({
  invokeCommand: mockInvoke,
}));

describe("useShortcutsStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    vi.resetAllMocks();
    mockStore.stored = undefined;
    mockStore.get.mockImplementation(async () => mockStore.stored);
    mockStore.set.mockImplementation(async (_key, val) => {
      mockStore.stored = val;
    });
    mockStore.save.mockResolvedValue(undefined);
    mockInvoke.mockResolvedValue({
      shortcut: "CommandOrControl+Shift+Space",
      registered: true,
      error: null,
    } satisfies DictationShortcutStatus);
  });

  it("loads stored custom shortcuts", async () => {
    mockStore.stored = {
      schemaVersion: 1,
      value: {
        custom: {
          "shell.commandPalette": "$mod+P",
        },
      },
    };
    const store = useShortcutsStore();
    await store.load();

    expect(store.customShortcuts["shell.commandPalette"]).toBe("$mod+P");
    expect(store.getEffectiveShortcut("shell.commandPalette", "$mod+K")).toBe("$mod+P");
    expect(store.isCustomized("shell.commandPalette")).toBe(true);
  });

  it("sets and persists a custom shortcut", async () => {
    const store = useShortcutsStore();
    await store.load();
    await store.setCustomShortcut("shell.commandPalette", "$mod+P");

    expect(store.customShortcuts["shell.commandPalette"]).toBe("$mod+P");
    expect(mockStore.set).toHaveBeenCalled();
  });

  it("resets a custom shortcut", async () => {
    const store = useShortcutsStore();
    await store.load();
    await store.setCustomShortcut("shell.commandPalette", "$mod+P");
    expect(store.isCustomized("shell.commandPalette")).toBe(true);

    await store.resetCustomShortcut("shell.commandPalette");
    expect(store.isCustomized("shell.commandPalette")).toBe(false);
    expect(store.getEffectiveShortcut("shell.commandPalette", "$mod+K")).toBe("$mod+K");
  });

  it("resets all custom shortcuts", async () => {
    const store = useShortcutsStore();
    await store.load();
    await store.setCustomShortcut("cmd1", "$mod+1");
    await store.setCustomShortcut("cmd2", "$mod+2");

    await store.resetAll();
    expect(store.isCustomized("cmd1")).toBe(false);
    expect(store.isCustomized("cmd2")).toBe(false);
  });
});
