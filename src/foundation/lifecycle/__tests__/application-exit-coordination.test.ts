import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  listen: vi.fn<(name: string, handler: () => Promise<void>) => Promise<() => void>>(),
  close: vi.fn<(handler: (event: { preventDefault(): void }) => Promise<void>) => Promise<() => void>>(),
  hide: vi.fn<() => Promise<void>>(),
  invoke: vi.fn<(command: string, payload?: Record<string, unknown>) => Promise<void>>(),
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ onCloseRequested: mocks.close, hide: mocks.hide }),
}));
vi.mock("@tauri-apps/plugin-log", () => ({ error: vi.fn() }));
vi.mock("../../ipc/invoke-command", () => ({ invokeCommand: mocks.invoke }));

let exit: () => Promise<void>;
let close: (event: { preventDefault(): void }) => Promise<void>;
beforeEach(() => {
  vi.resetModules();
  vi.resetAllMocks();
  mocks.listen.mockImplementation(async (_name, handler) => {
    exit = handler;
    return () => undefined;
  });
  mocks.close.mockImplementation(async (handler) => {
    close = handler;
    return () => undefined;
  });
  mocks.invoke.mockResolvedValue(undefined);
});

describe("quit, hide and update installation exclusion", () => {
  it("rejects ordinary quit without saving twice during an update transition", async () => {
    const coordinator = await import("../window-close-coordinator");
    const lifecycle = await import("../application-transition");
    const save = vi.fn<() => Promise<void>>(async () => undefined);
    coordinator.registerWindowCloseCommit(save);
    await coordinator.initializeExitCoordination();
    await lifecycle.runApplicationTransition(exit);
    expect(mocks.invoke).toHaveBeenLastCalledWith("plugin:desktop|decide_app_exit_request", {
      approved: false,
    });
    expect(save).not.toHaveBeenCalled();
  });

  it("does not hide the installation modal when the native close button is pressed", async () => {
    const coordinator = await import("../window-close-coordinator");
    const lifecycle = await import("../application-transition");
    await coordinator.initializeExitCoordination();
    const event = { preventDefault: vi.fn() };
    await lifecycle.runApplicationTransition(() => close(event));
    expect(event.preventDefault).toHaveBeenCalledOnce();
    expect(mocks.hide).not.toHaveBeenCalled();
  });

  it("keeps the transition locked until quit saves finish and then releases it", async () => {
    const coordinator = await import("../window-close-coordinator");
    const lifecycle = await import("../application-transition");
    let finish!: () => void;
    coordinator.registerWindowCloseCommit(
      () => new Promise<void>((resolve) => { finish = resolve; }),
    );
    await coordinator.initializeExitCoordination();
    const quitting = exit();
    await vi.waitFor(() => expect(finish).toBeDefined());
    expect(lifecycle.isApplicationTransitioning()).toBe(true);
    finish();
    await quitting;
    expect(lifecycle.isApplicationTransitioning()).toBe(false);
    expect(mocks.invoke).toHaveBeenLastCalledWith("plugin:desktop|decide_app_exit_request", {
      approved: true,
    });
  });
});
