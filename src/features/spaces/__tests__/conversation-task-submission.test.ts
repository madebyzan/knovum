import { beforeEach, describe, expect, it, vi } from "vitest";
import { ref, shallowRef } from "vue";

const ipc = vi.hoisted(() => ({
  getConversationTaskError: vi.fn<(runId: string) => Promise<string | null>>(),
  startConversationTask: vi.fn<
    () => Promise<{
      accepted: { taskId: string; runId: string; conversationId: string; branchId: string };
      settled: Promise<unknown>;
    }>
  >(),
}));
const observation = vi.hoisted(() => ({ observeConversationRun: vi.fn<() => void>() }));

vi.mock("../conversation/conversation-ipc", () => ipc);
vi.mock("../conversation/conversation-run-observation", () => observation);

import { createConversationTaskSubmission } from "../conversation/conversation-task-submission";

describe("conversation task settlement", () => {
  beforeEach(() => vi.clearAllMocks());

  it("shows a channel settlement failure without treating it as a durable run state", async () => {
    ipc.startConversationTask.mockResolvedValue({
      accepted: {
        taskId: "task",
        runId: "run",
        conversationId: "conversation",
        branchId: "branch",
      },
      settled: Promise.resolve({
        type: "finished",
        state: "settlement_failed",
        error: "final state could not be read",
      }),
    });
    const state = {
      activeRun: shallowRef(),
      activeRunId: shallowRef<string>(),
      activeConversationId: shallowRef("conversation"),
      activeSpaceId: shallowRef<string>(),
      taskState: shallowRef<string>(),
      loading: shallowRef(false),
      error: shallowRef<string>(),
      streamedText: shallowRef(""),
      activityRevision: shallowRef(0),
    };
    const restoreDraft = vi.fn<(request: unknown) => void>();
    const submission = createConversationTaskSubmission({
      input: {
        extensions: ref({ skillIds: [], connections: [] }),
        spaceId: ref("space"),
        selectedConversationId: ref("conversation"),
        conversation: ref(),
        prompt: ref("prompt"),
        target: ref({ type: "agent", runtimeKind: "codex" }),
        attachments: ref([]),
        visibleAttachments: ref([]),
        messages: shallowRef([]),
        clearDraft: vi.fn<(request: unknown) => void>(),
        restoreDraft,
      },
      state,
      accepted: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
      settled: vi.fn<() => Promise<void>>().mockRejectedValue(new Error("projection database unavailable")),
      restoreForSpace: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
    });

    await submission.send();

    expect(state.taskState.value).toBe("settlement_failed");
    expect(state.error.value).toContain("final state could not be read");
    expect(state.loading.value).toBe(false);
    expect(state.activeRunId.value).toBeUndefined();
    expect(ipc.getConversationTaskError).not.toHaveBeenCalled();
    expect(restoreDraft).not.toHaveBeenCalled();
  });
});
