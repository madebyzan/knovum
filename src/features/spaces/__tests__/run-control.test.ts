import { beforeEach, describe, expect, it, vi } from "vitest";

const bridge = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args?: unknown) => Promise<unknown>>(),
}));
const notifications = vi.hoisted(() => ({
  warning: vi.fn<(message: string) => void>(),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: bridge.invoke }));
vi.mock("vue-sonner", () => ({ toast: { warning: notifications.warning } }));

import { cancelConversationTask, pauseConversationTask } from "../conversation/conversation-ipc";
import { cancelResourceRun } from "../runs/resource-run-observation";
import { requestRunControl } from "../runs/run-control";

beforeEach(() => {
  vi.clearAllMocks();
});

describe("run control receipts", () => {
  it("resolves an accepted request and reports every interruption warning", async () => {
    bridge.invoke.mockResolvedValue({
      interruptionErrors: [
        {
          code: "run_control_ai_interrupt_failed",
          message: "Provider did not acknowledge cancellation",
        },
        {
          code: "run_control_action_interrupt_failed",
          message: "Worker cleanup could not be interrupted",
        },
      ],
    });

    await expect(requestRunControl("cancel", "run-1")).resolves.toBeUndefined();

    expect(bridge.invoke).toHaveBeenCalledExactlyOnceWith("plugin:spaces|cancel_run", {
      runId: "run-1",
    });
    expect(notifications.warning).toHaveBeenNthCalledWith(1, "Provider did not acknowledge cancellation");
    expect(notifications.warning).toHaveBeenNthCalledWith(2, "Worker cleanup could not be interrupted");
  });

  it("propagates a durable rejection without reporting an interruption warning", async () => {
    bridge.invoke.mockRejectedValue({
      code: "run_control_rejected",
      message: "Control intent was not saved",
    });

    await expect(requestRunControl("pause", "run-2")).rejects.toMatchObject({
      message: "Control intent was not saved",
      cause: { code: "run_control_rejected", message: "Control intent was not saved" },
    });
    expect(notifications.warning).not.toHaveBeenCalled();
  });

  it("routes resource and conversation controls through the finite command owner", async () => {
    bridge.invoke.mockResolvedValue({ interruptionErrors: [] });

    await cancelResourceRun("resource-run");
    await cancelConversationTask("conversation-run");
    await pauseConversationTask("conversation-run");

    expect(bridge.invoke).toHaveBeenNthCalledWith(1, "plugin:spaces|cancel_run", {
      runId: "resource-run",
    });
    expect(bridge.invoke).toHaveBeenNthCalledWith(2, "plugin:spaces|cancel_run", {
      runId: "conversation-run",
    });
    expect(bridge.invoke).toHaveBeenNthCalledWith(3, "plugin:spaces|pause_run", {
      runId: "conversation-run",
    });
  });
});
