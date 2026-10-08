import { createPinia, setActivePinia } from "pinia";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useTaskStore } from "../../tasks/task-store";
import { runPluginOperation } from "../plugin-operation";

const mocks = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, payload?: Record<string, unknown>) => Promise<unknown>>(),
  progress: undefined as ((stage: string) => void) | undefined,
}));
vi.mock("../../ipc/invoke-command", () => ({ invokeCommand: mocks.invoke }));
vi.mock("@tauri-apps/api/core", () => ({
  Channel: function (listener: (stage: string) => void) {
    mocks.progress = listener;
  },
}));

describe("plugin operations in Task Center", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mocks.invoke.mockReset();
    vi.useFakeTimers();
  });
  afterEach(() => vi.useRealTimers());

  it("uses one host job identity and records committed completion", async () => {
    mocks.invoke.mockImplementation(async () => {
      mocks.progress!("reading");
      mocks.progress!("committing");
      expect(useTaskStore().tasks[0]?.cancellable).toBe(false);
      return { id: "installation" };
    });
    const installation = await runPluginOperation("install_plugin", "extensions.taskInstall", {
      source: "source",
    });
    const task = useTaskStore().tasks[0]!;
    expect(installation).toEqual({ id: "installation" });
    expect(task.status).toBe("succeeded");
    expect(mocks.invoke).toHaveBeenCalledWith(
      "plugin:extensions|install_plugin",
      expect.objectContaining({ jobId: task.id, source: "source" }),
    );
  });

  it("keeps cancellation pending until the host acknowledges the terminal outcome", async () => {
    let rejectOperation!: (reason: unknown) => void;
    mocks.invoke.mockImplementation((command: string) =>
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
    rejectOperation({ code: "cancelled", message: "Operation interrupted by the user" });
    expect(await rejected).toBeInstanceOf(Error);
    expect(tasks.tasks[0]?.status).toBe("cancelled");
  });

  it("records a failed download without treating it as cancellation", async () => {
    mocks.invoke.mockRejectedValue(new Error("source unreachable"));
    await expect(runPluginOperation("inspect_plugin_source", "extensions.taskInspect", {})).rejects.toThrow(
      "source unreachable",
    );
    expect(useTaskStore().tasks[0]).toMatchObject({ status: "failed", statusMessage: "source unreachable" });
  });
});
