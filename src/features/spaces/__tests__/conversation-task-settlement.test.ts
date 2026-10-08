import { beforeEach, describe, expect, it, vi } from "vitest";

const ipc = vi.hoisted(() => ({
  getConversationTaskError: vi.fn<(runId: string) => Promise<string | null>>(),
}));
vi.mock("../conversation/conversation-ipc", () => ipc);

import { settleConversationTaskResult } from "../conversation/conversation-task-settlement";

const accepted = {
  taskId: "task",
  runId: "run",
  conversationId: "conversation",
  branchId: "branch",
};

describe("conversation task result settlement", () => {
  beforeEach(() => vi.clearAllMocks());

  it("preserves independent channel and projection failures", async () => {
    const projectionError = new Error("projection database unavailable");
    const settlement = settleConversationTaskResult({
      accepted,
      outcome: {
        type: "finished",
        state: "settlement_failed",
        error: "final state could not be read",
      },
      spaceId: "space",
      project: vi.fn<() => Promise<void>>().mockRejectedValue(projectionError),
    });

    const error = await settlement.catch((reason: unknown) => reason);
    expect(error).toBeInstanceOf(AggregateError);
    expect(error).toMatchObject({
      message: "final state could not be read",
      cause: expect.objectContaining({ message: "final state could not be read" }),
    });
    expect((error as AggregateError).errors).toEqual([
      expect.objectContaining({ message: "final state could not be read" }),
      projectionError,
    ]);
    expect(ipc.getConversationTaskError).not.toHaveBeenCalled();
  });

  it("uses the persisted diagnostic for an ordinary failed run", async () => {
    ipc.getConversationTaskError.mockResolvedValue("provider rejected request");

    await expect(
      settleConversationTaskResult({
        accepted,
        outcome: { type: "finished", state: "failed", error: null },
        spaceId: "space",
        project: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
      }),
    ).rejects.toThrow("provider rejected request");
  });
});
