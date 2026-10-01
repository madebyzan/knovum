import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import type { UpdateSnapshot } from "../../../shared/contracts/generated/UpdateSnapshot";
import { useUpdateStore } from "../update-store";

type Observer = { onmessage: (value: unknown) => void };
type CommandArguments = { onEvent?: Observer; version?: string };

const mocks = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args?: CommandArguments) => Promise<unknown>>(),
  commit: vi.fn<(install: () => Promise<void>) => Promise<void>>(),
  mirror: vi.fn<(snapshot: UpdateSnapshot) => void>(),
  tasks: { running: [] as { id: string }[] },
  toast: {
    info: vi.fn<(message: string) => void>(),
    success: vi.fn<(message: string, options?: unknown) => void>(),
    error: vi.fn<(message: string) => void>(),
  },
}));

vi.mock("@tauri-apps/api/core", () => ({
  Channel: class {
    constructor(public onmessage: (value: unknown) => void) {}
  },
}));
vi.mock("@tauri-apps/plugin-log", () => ({
  error: vi.fn<(message: string) => Promise<void>>().mockResolvedValue(undefined),
}));
vi.mock("vue-sonner", () => ({ toast: mocks.toast }));
vi.mock("../../ipc/invoke-command", () => ({ invokeCommand: mocks.invoke }));
vi.mock("../../i18n/i18n", () => ({ i18n: { global: { t: (key: string) => key } } }));
vi.mock("../../tasks/task-store", () => ({ useTaskStore: () => mocks.tasks }));
vi.mock("../../lifecycle/window-close-coordinator", () => ({ commitApplicationRestart: mocks.commit }));
vi.mock("../update-task", () => ({ createUpdateTask: () => mocks.mirror }));

function snapshot(phase: UpdateSnapshot["phase"] = "ready", revision = 1): UpdateSnapshot {
  return {
    revision,
    configured: true,
    phase,
    update: { version: "0.0.2", currentVersion: "0.0.1", notes: null },
    downloaded: 12,
    total: 12,
    failure: null,
  };
}

let observer: { onmessage: (value: unknown) => void } | undefined;

beforeEach(() => {
  vi.clearAllMocks();
  setActivePinia(createPinia());
  mocks.tasks.running = [];
  observer = undefined;
  mocks.commit.mockImplementation(async (install: () => Promise<void>) => install());
  mocks.invoke.mockImplementation(async (command, args) => {
    if (command === "observe_updates") {
      observer = args?.onEvent;
      return snapshot();
    }
    return undefined;
  });
});

describe("application update session", () => {
  it("survives settings pane remounts as a single Pinia session", async () => {
    const first = useUpdateStore();
    await first.connect();
    const second = useUpdateStore();
    expect(second).toBe(first);
    expect(second.snapshot?.update?.version).toBe("0.0.2");
  });

  it("installs the displayed version without re-checking the feed", async () => {
    const store = useUpdateStore();
    await store.connect();
    await store.install();
    expect(mocks.commit).toHaveBeenCalledOnce();
    expect(mocks.invoke).toHaveBeenCalledWith("install_update", { version: "0.0.2" });
    expect(mocks.invoke.mock.calls.some(([command]) => command === "check_for_update")).toBe(false);
  });

  it("does not invoke the installer after a failed save barrier", async () => {
    mocks.commit.mockRejectedValueOnce(new Error("disk full"));
    const store = useUpdateStore();
    await store.connect();
    await store.install();
    expect(mocks.invoke.mock.calls.some(([command]) => command === "install_update")).toBe(false);
    expect(store.blocking).toBe(false);
    expect(mocks.toast.error).toHaveBeenCalled();
  });

  it("requires background tasks to finish before saving and installing", async () => {
    mocks.tasks.running = [{ id: "transcription" }];
    const store = useUpdateStore();
    await store.connect();
    await store.install();
    expect(mocks.commit).not.toHaveBeenCalled();
    expect(mocks.toast.error).toHaveBeenCalledWith("settings.updates.install.runningTasks");
  });

  it("checks again for work started by save callbacks", async () => {
    mocks.commit.mockImplementationOnce(async (install: () => Promise<void>) => {
      mocks.tasks.running = [{ id: "save-triggered-work" }];
      await install();
    });
    const store = useUpdateStore();
    await store.connect();
    await store.install();
    expect(mocks.invoke.mock.calls.some(([command]) => command === "install_update")).toBe(false);
  });

  it("never regresses to a stale snapshot after a newer channel event", async () => {
    mocks.invoke.mockImplementationOnce(async (_command, args) => {
      args?.onEvent?.onmessage(snapshot("ready", 3));
      return snapshot("available", 2);
    });
    const store = useUpdateStore();
    await store.connect();
    expect(store.phase).toBe("ready");
    expect(store.snapshot?.revision).toBe(3);
  });

  it("does not notify repeatedly when the same available version is observed", async () => {
    const store = useUpdateStore();
    await store.connect();
    observer?.onmessage(snapshot("available", 2));
    observer?.onmessage(snapshot("available", 3));
    expect(mocks.toast.info).toHaveBeenCalledOnce();
  });

  it("rejects malformed IPC state instead of coercing unknown phases", async () => {
    const store = useUpdateStore();
    await store.connect();
    observer?.onmessage({ ...snapshot(), phase: "unknown" });
    expect(store.disconnected).toBe(true);
    expect(store.phase).toBe("ready");
  });

  it("blocks a second user action while a check is in flight", async () => {
    const store = useUpdateStore();
    await store.connect();
    let finish!: (value: UpdateSnapshot) => void;
    mocks.invoke.mockImplementationOnce(
      () =>
        new Promise((resolve) => {
          finish = resolve;
        }),
    );
    const first = store.check();
    await Promise.resolve();
    await store.check();
    finish(snapshot("available", 2));
    await first;
    expect(mocks.invoke.mock.calls.filter(([command]) => command === "check_for_update")).toHaveLength(1);
  });
});
