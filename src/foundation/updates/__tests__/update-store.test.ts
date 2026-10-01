import { createPinia, setActivePinia } from "pinia";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { UpdateState } from "../../../shared/contracts/generated/UpdateState";
import { useTaskStore } from "../../tasks/task-store";
import { useUpdateStore } from "../update-store";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, payload?: Record<string, unknown>) => Promise<unknown>>(),
  listen: vi.fn<(event: string, handler: (event: { payload: unknown }) => void) => Promise<() => void>>(),
  stop: vi.fn(),
  flush: vi.fn<() => Promise<void>>(),
  log: vi.fn(),
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));
vi.mock("@tauri-apps/plugin-log", () => ({ error: mocks.log }));
vi.mock("../../ipc/invoke-command", () => ({ invokeCommand: mocks.invoke }));
vi.mock("../../lifecycle/window-close-coordinator", () => ({ flushWindowCloseCommits: mocks.flush }));

const release = { version: "1.2.3", currentVersion: "1.2.2", notes: "Reviewed update" };
let receive: (event: { payload: unknown }) => void;
function emit(revision: number, state: UpdateState) { receive({ payload: { revision, state } }); }
function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((complete) => { resolve = complete; });
  return { promise, resolve };
}

beforeEach(() => {
  vi.resetAllMocks();
  setActivePinia(createPinia());
  mocks.listen.mockImplementation(async (event, handler) => {
    expect(event).toBe("knovum://update-state");
    receive = handler;
    return mocks.stop;
  });
  mocks.invoke.mockResolvedValue({ revision: 0, state: { status: "idle" } });
  mocks.flush.mockResolvedValue(undefined);
});
afterEach(() => { useUpdateStore().$dispose(); useTaskStore().$dispose(); });

describe("application update ownership", () => {
  it("subscribes before reading state and ignores a stale initial response", async () => {
    const initial = deferred<unknown>();
    mocks.invoke.mockReturnValue(initial.promise);
    const updates = useUpdateStore();
    const connecting = updates.initialize();
    await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("get_update_state"));
    emit(3, { status: "available", release });
    initial.resolve({ revision: 1, state: { status: "current" } });
    await connecting;
    expect(updates.state).toEqual({ status: "available", release });
  });

  it("rejects malformed IPC events instead of changing the update state", async () => {
    const updates = useUpdateStore();
    await updates.initialize();
    receive({ payload: { revision: 2, state: { status: "ready", release: { version: "1.2.3" } } } });
    expect(updates.state?.status).toBe("idle");
    expect(mocks.log).toHaveBeenCalledOnce();
  });

  it("releases a failed connection before an explicit retry", async () => {
    const updates = useUpdateStore();
    mocks.invoke.mockRejectedValueOnce(new Error("IPC unavailable"));
    await expect(updates.initialize()).rejects.toThrow("IPC unavailable");
    expect(mocks.stop).toHaveBeenCalledOnce();
    await updates.initialize();
    expect(mocks.listen).toHaveBeenCalledTimes(2);
  });

  it("shares one request across views and downloads only the displayed version", async () => {
    const updates = useUpdateStore();
    await updates.initialize();
    emit(1, { status: "available", release });
    const downloaded = deferred<unknown>();
    mocks.invoke.mockReturnValue(downloaded.promise);
    const operation = updates.download();
    await vi.waitFor(() => expect(mocks.invoke).toHaveBeenCalledWith("download_update", { version: "1.2.3" }));
    expect(useUpdateStore()).toBe(updates);
    await expect(useUpdateStore().download()).rejects.toThrow("already running");
    downloaded.resolve({ revision: 4, state: { status: "ready", release } });
    await operation;
    expect(updates.canInstall).toBe(true);
  });

  it("accepts an authoritative failure response even without the corresponding event", async () => {
    const updates = useUpdateStore();
    await updates.initialize();
    mocks.invoke.mockResolvedValue({ revision: 2, state: { status: "failed", reason: "check" } });
    await updates.check();
    expect(updates.state).toEqual({ status: "failed", reason: "check" });
    expect(updates.requestPending).toBe(false);
  });

  it("does not invoke the native installer if saving fails", async () => {
    const updates = useUpdateStore();
    await updates.initialize();
    emit(1, { status: "ready", release });
    mocks.flush.mockRejectedValue(new Error("Save failed"));
    await expect(updates.install()).rejects.toThrow("Save failed");
    expect(mocks.invoke).not.toHaveBeenCalledWith("install_update", expect.anything());
    expect(updates.installing).toBe(false);
    expect(updates.state?.status).toBe("ready");
  });

  it("blocks installation while another task is running", async () => {
    const updates = useUpdateStore();
    await updates.initialize();
    emit(1, { status: "ready", release });
    useTaskStore().start("Transcription");
    expect(updates.canInstall).toBe(false);
    await expect(updates.install()).rejects.toThrow("background tasks");
    expect(mocks.flush).not.toHaveBeenCalled();
  });

  it("waits for all save commits and checks for newly started jobs before installing", async () => {
    const updates = useUpdateStore();
    await updates.initialize();
    emit(1, { status: "ready", release });
    const saving = deferred<void>();
    mocks.flush.mockReturnValue(saving.promise);
    const operation = updates.install();
    expect(updates.installing).toBe(true);
    useTaskStore().start("Import");
    saving.resolve(undefined);
    await expect(operation).rejects.toThrow("started while preparing");
    expect(mocks.invoke).not.toHaveBeenCalledWith("install_update", expect.anything());
  });

  it("installs the verified version after saving and releases the modal on native failure", async () => {
    const updates = useUpdateStore();
    await updates.initialize();
    emit(1, { status: "ready", release });
    mocks.invoke.mockResolvedValue({ revision: 3, state: { status: "failed", reason: "install" } });
    await updates.install();
    expect(mocks.flush).toHaveBeenCalledOnce();
    expect(mocks.invoke).toHaveBeenLastCalledWith("install_update", { version: "1.2.3" });
    expect(updates.state).toEqual({ status: "failed", reason: "install" });
    expect(updates.installing).toBe(false);
  });
});
