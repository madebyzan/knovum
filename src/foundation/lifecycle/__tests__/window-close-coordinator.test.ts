import { beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn(async () => vi.fn()) }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ onCloseRequested: vi.fn(async () => vi.fn()), hide: vi.fn() }),
}));
vi.mock("@tauri-apps/plugin-log", () => ({ error: vi.fn() }));
vi.mock("../../ipc/invoke-command", () => ({ invokeCommand: vi.fn(async () => undefined) }));

function deferred() {
  let resolve!: () => void;
  const promise = new Promise<void>((complete) => { resolve = complete; });
  return { promise, resolve };
}
beforeEach(() => { vi.resetModules(); });

describe("shared save barrier for quit and update installation", () => {
  it("coalesces concurrent requests and starts each save exactly once", async () => {
    const coordinator = await import("../window-close-coordinator");
    const save = deferred();
    const commit = vi.fn(() => save.promise);
    coordinator.registerWindowCloseCommit(commit);
    const first = coordinator.flushWindowCloseCommits();
    const second = coordinator.flushWindowCloseCommits();
    expect(first).toBe(second);
    await Promise.resolve();
    expect(commit).toHaveBeenCalledOnce();
    save.resolve();
    await first;
  });

  it("waits for slower saves even when another save rejects", async () => {
    const coordinator = await import("../window-close-coordinator");
    const slow = deferred();
    coordinator.registerWindowCloseCommit(async () => { throw new Error("Disk full"); });
    coordinator.registerWindowCloseCommit(() => slow.promise);
    let settled = false;
    const outcome = coordinator.flushWindowCloseCommits().catch((error: unknown) => {
      settled = true;
      return error;
    });
    await Promise.resolve();
    await Promise.resolve();
    expect(settled).toBe(false);
    slow.resolve();
    expect(await outcome).toBeInstanceOf(AggregateError);
  });

  it("captures synchronous failures, saves other owners and allows an explicit retry", async () => {
    const coordinator = await import("../window-close-coordinator");
    const remove = coordinator.registerWindowCloseCommit(() => { throw new Error("Synchronous save failure"); });
    const healthy = vi.fn(async () => undefined);
    coordinator.registerWindowCloseCommit(healthy);
    await expect(coordinator.flushWindowCloseCommits()).rejects.toBeInstanceOf(AggregateError);
    expect(healthy).toHaveBeenCalledOnce();
    remove();
    await coordinator.flushWindowCloseCommits();
    expect(healthy).toHaveBeenCalledTimes(2);
  });
});
