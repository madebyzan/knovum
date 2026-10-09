import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  listen:
    vi.fn<
      (
        name: string,
        handler: (event: { payload: { requestId: number } }) => Promise<void>,
      ) => Promise<() => void>
    >(),
  close: vi.fn<(handler: (event: { preventDefault(): void }) => Promise<void>) => Promise<() => void>>(),
  invoke: vi.fn<(command: string, payload?: Record<string, unknown>) => Promise<unknown>>(),
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: mocks.listen }));
vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({ onCloseRequested: mocks.close }),
}));
vi.mock("@tauri-apps/plugin-log", () => ({ error: vi.fn<(message: string) => void>() }));
vi.mock("../../ipc/invoke-command", () => ({ invokeCommand: mocks.invoke }));

let exit: (requestId?: number) => Promise<void>;
let close: (event: { preventDefault(): void }) => Promise<void>;
beforeEach(() => {
  vi.resetModules();
  vi.resetAllMocks();
  mocks.listen.mockImplementation(async (_name, handler) => {
    exit = (requestId = 1) => handler({ payload: { requestId } });
    return () => undefined;
  });
  mocks.close.mockImplementation(async (handler) => {
    close = handler;
    return () => undefined;
  });
  mocks.invoke.mockImplementation(async (command) =>
    command === "plugin:desktop|begin_window_close" ? 1 : undefined,
  );
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
      requestId: 1,
      approved: false,
    });
    expect(save).not.toHaveBeenCalled();
  });

  it("does not hide the installation modal when the native close button is pressed", async () => {
    const coordinator = await import("../window-close-coordinator");
    const lifecycle = await import("../application-transition");
    await coordinator.initializeExitCoordination();
    const event = { preventDefault: vi.fn<() => void>() };
    await lifecycle.runApplicationTransition(() => close(event));
    expect(event.preventDefault).toHaveBeenCalledOnce();
    expect(mocks.invoke).toHaveBeenLastCalledWith("plugin:desktop|complete_window_close", {
      requestId: 1,
      hide: false,
    });
  });

  it("keeps the transition locked until quit saves finish and then releases it", async () => {
    const coordinator = await import("../window-close-coordinator");
    const lifecycle = await import("../application-transition");
    let finish!: () => void;
    coordinator.registerWindowCloseCommit(
      () =>
        new Promise<void>((resolve) => {
          finish = resolve;
        }),
    );
    await coordinator.initializeExitCoordination();
    const quitting = exit();
    await vi.waitFor(() => expect(finish).toBeDefined());
    expect(lifecycle.isApplicationTransitioning()).toBe(true);
    finish();
    await quitting;
    expect(lifecycle.isApplicationTransitioning()).toBe(false);
    expect(mocks.invoke).toHaveBeenLastCalledWith("plugin:desktop|decide_app_exit_request", {
      requestId: 1,
      approved: true,
    });
  });

  it("latches a window close before the native begin request resolves", async () => {
    let resolveBegin!: (requestId: number) => void;
    const begin = new Promise<number>((resolve) => {
      resolveBegin = resolve;
    });
    mocks.invoke.mockImplementation(async (command) =>
      command === "plugin:desktop|begin_window_close" ? begin : undefined,
    );
    const coordinator = await import("../window-close-coordinator");
    await coordinator.initializeExitCoordination();
    const firstEvent = { preventDefault: vi.fn<() => void>() };
    const repeatedEvent = { preventDefault: vi.fn<() => void>() };

    const firstClose = close(firstEvent);
    const repeatedClose = close(repeatedEvent);
    expect(firstEvent.preventDefault).toHaveBeenCalledOnce();
    expect(repeatedEvent.preventDefault).toHaveBeenCalledOnce();
    expect(
      mocks.invoke.mock.calls.filter(([command]) => command === "plugin:desktop|begin_window_close"),
    ).toHaveLength(1);

    resolveBegin(17);
    await Promise.all([firstClose, repeatedClose]);
  });

  it("latches an exit request before its acknowledgement resolves", async () => {
    let resolveAcknowledgement!: () => void;
    const acknowledgement = new Promise<void>((resolve) => {
      resolveAcknowledgement = resolve;
    });
    mocks.invoke.mockImplementation(async (command) =>
      command === "plugin:desktop|acknowledge_app_exit_request" ? acknowledgement : undefined,
    );
    const coordinator = await import("../window-close-coordinator");
    await coordinator.initializeExitCoordination();

    const firstExit = exit(23);
    const repeatedExit = exit(23);
    await Promise.resolve();
    expect(
      mocks.invoke.mock.calls.filter(
        ([command]) => command === "plugin:desktop|acknowledge_app_exit_request",
      ),
    ).toHaveLength(1);

    resolveAcknowledgement();
    await Promise.all([firstExit, repeatedExit]);
  });
});
