import { createPinia, setActivePinia } from "pinia";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useTaskStore } from "../../tasks/task-store";
import { runPluginOperation } from "../plugin-operation";

const native = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, payload?: Record<string, unknown>) => Promise<unknown>>(),
  progress: undefined as ((stage: string) => void) | undefined,
}));
vi.mock("../../ipc/invoke-command", () => ({ invokeCommand: native.invoke }));
vi.mock("@tauri-apps/api/core", () => ({
  Channel: function (listener: (stage: string) => void) {
    native.progress = listener;
  },
}));

describe("plugin operations in Task Center", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    native.invoke.mockReset();
    vi.useFakeTimers();
  });
  afterEach(() => vi.useRealTimers());

  it("uses one host job identity and records committed completion", async () => {
    native.invoke.mockImplementation(async () => {
      native.progress!("reading");
      native.progress!("committing");
      expect(useTaskStore().tasks[0]?.cancellable).toBe(false);
      return { id: "installation" };
    });
    const result = await runPluginOperation("install_plugin", "extensions.taskInstall", { source: "source" });
    const task = useTaskStore().tasks[0]!;
    expect(result).toEqual({ id: "installation" });
    expect(task.status).toBe("succeeded");
    expect(native.invoke).toHaveBeenCalledWith(
      "plugin:extensions|install_plugin",
      expect.objectContaining({ jobId: task.id, source: "source" }),
    );
  });

  it("keeps cancellation pending until the host acknowledges the terminal outcome", async () => {
    let rejectOperation!: (reason: Error) => void;
    native.invoke.mockImplementation((command: string) =>
      command.endsWith("cancel_plugin_operation")
        ? Promise.resolve()
        : new Promise((_resolve, reject) => {
            rejectOperation = reject;
          }),
    );
    const completion = runPluginOperation("inspect_plugin_source", "extensions.taskInspect", {});
    const rejected = completion.catch((error: Error) => error);
    const tasks = useTaskStore();
    await tasks.cancel(tasks.tasks[0]!.id);
    expect(tasks.tasks[0]?.status).toBe("running");
    rejectOperation(new Error("plugin-operation-cancelled"));
    expect(await rejected).toBeInstanceOf(Error);
    expect(tasks.tasks[0]?.status).toBe("cancelled");
  });

  it("records a failed download without treating it as cancellation", async () => {
    native.invoke.mockRejectedValue(new Error("source unreachable"));
    await expect(runPluginOperation("inspect_plugin_source", "extensions.taskInspect", {})).rejects.toThrow(
      "source unreachable",
    );
    expect(useTaskStore().tasks[0]).toMatchObject({ status: "failed", statusMessage: "source unreachable" });
  });
});
