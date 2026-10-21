import { beforeEach, describe, expect, it, vi } from "vitest";

import { deferred } from "../../../shared/__tests__/support/deferred";

vi.mock("../../i18n/instance", () => ({ i18n: { global: { t: (key: string) => key } } }));
vi.mock("vue-sonner", () => ({ toast: { error: vi.fn<(message: string) => void>() } }));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn<() => Promise<() => void>>(async () => vi.fn<() => void>()),
}));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    onCloseRequested: vi.fn<() => Promise<() => void>>(async () => vi.fn<() => void>()),
    hide: vi.fn<() => Promise<void>>(),
  }),
}));
vi.mock("@tauri-apps/plugin-log", () => ({ error: vi.fn<(message: string) => Promise<void>>() }));
vi.mock("../../ipc/invoke-command", () => ({
  invokeCommand: vi.fn<() => Promise<void>>(async () => undefined),
}));

beforeEach(() => {
  vi.resetModules();
});

describe("shared save barrier for quit and update installation", () => {
  it("coalesces concurrent requests and starts each save exactly once", async () => {
    const coordinator = await import("../window-close-coordinator");
    const save = deferred();
    const commit = vi.fn<() => Promise<void>>(() => save.promise);
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
    coordinator.registerWindowCloseCommit(async () => {
      throw new Error("Disk full");
    });
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
    const remove = coordinator.registerWindowCloseCommit(() => {
      throw new Error("Synchronous save failure");
    });
    const healthy = vi.fn<() => Promise<void>>(async () => undefined);
    coordinator.registerWindowCloseCommit(healthy);
    await expect(coordinator.flushWindowCloseCommits()).rejects.toBeInstanceOf(AggregateError);
    expect(healthy).toHaveBeenCalledOnce();
    remove();
    await coordinator.flushWindowCloseCommits();
    expect(healthy).toHaveBeenCalledTimes(2);
  });
});
