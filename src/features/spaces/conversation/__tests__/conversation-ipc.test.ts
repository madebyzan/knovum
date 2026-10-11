import { beforeEach, describe, expect, it, vi } from "vitest";

const bridge = vi.hoisted(() => ({
  invoke: vi.fn<(command: string, args?: unknown) => Promise<unknown>>(),
  receive: undefined as ((update: unknown) => void) | undefined,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: bridge.invoke,
  Channel: vi.fn<(receive: (update: unknown) => void) => void>(function channel(receive) {
    bridge.receive = receive;
  }),
}));

import { startConversationTask } from "../conversation-ipc";

describe("conversation task channel settlement", () => {
  beforeEach(() => {
    bridge.invoke.mockReset();
    bridge.receive = undefined;
  });

  it("settles with an explicit settlement failure diagnostic", async () => {
    bridge.invoke.mockResolvedValue({
      taskId: "task",
      runId: "run",
      conversationId: "conversation",
      branchId: "branch",
    });
    const task = await startConversationTask(
      {
        spaceId: "space",
        prompt: "prompt",
        target: { type: "agent", runtimeKind: "codex" },
        attachments: [],
        extensions: { pluginIds: [], skillIds: [], connections: [] },
      },
      vi.fn<(update: unknown) => void>(),
    );
    const finished = {
      type: "finished" as const,
      state: "settlement_failed" as const,
      error: "final state could not be read",
    };

    bridge.receive?.(finished);

    await expect(task.settled).resolves.toEqual(finished);
  });
});
