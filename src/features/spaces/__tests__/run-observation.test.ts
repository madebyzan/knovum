import { afterEach, describe, expect, it, vi } from "vitest";

const bridge = vi.hoisted(() => ({
  invoke: vi.fn<(...args: unknown[]) => Promise<void>>(),
  update: null as null | ((run: unknown) => void),
}));
vi.mock("@tauri-apps/api/core", () => ({
  invoke: bridge.invoke,
  Channel: function (callback: (run: unknown) => void) {
    bridge.update = callback;
  },
}));
import { observeRun } from "../runs/run-observation";

const observer = () => ({
  update: vi.fn<(run: unknown) => void>(),
  error: vi.fn<(cause: unknown) => void>(),
  finished: vi.fn<() => void>(),
});
afterEach(() => vi.clearAllMocks());

describe("run observation ownership", () => {
  it("shares one IPC subscription between policies and releases it at completion", async () => {
    let complete!: () => void;
    bridge.invoke.mockReturnValue(
      new Promise<void>((resolve) => {
        complete = resolve;
      }),
    );
    const conversation = observer();
    const workflow = observer();
    observeRun("shared-run", conversation);
    observeRun("shared-run", workflow);
    expect(bridge.invoke).toHaveBeenCalledTimes(1);
    const run = { id: "shared-run", state: "running" };
    bridge.update!(run);
    expect(conversation.update).toHaveBeenCalledWith(run);
    expect(workflow.update).toHaveBeenCalledWith(run);
    complete();
    await vi.waitFor(() => expect(workflow.finished).toHaveBeenCalledOnce());
    expect(conversation.finished).toHaveBeenCalledOnce();
    observeRun("shared-run", observer());
    expect(bridge.invoke).toHaveBeenCalledTimes(2);
    complete();
    await vi.waitFor(() => expect(bridge.invoke).toHaveBeenCalledTimes(2));
  });

  it("reports IPC failure to every observer before releasing the subscription", async () => {
    const cause = new Error("connection lost");
    bridge.invoke.mockRejectedValue(cause);
    const first = observer();
    const second = observer();
    observeRun("failed-run", first);
    observeRun("failed-run", second);
    await vi.waitFor(() => expect(second.finished).toHaveBeenCalledOnce());
    expect(first.error).toHaveBeenCalledWith(cause);
    expect(second.error).toHaveBeenCalledWith(cause);
  });
});
