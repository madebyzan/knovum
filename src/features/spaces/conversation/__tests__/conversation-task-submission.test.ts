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

vi.mock("../conversation-ipc", () => ipc);
vi.mock("../conversation-run-observation", () => observation);

import { createConversationTaskSubmission } from "../conversation-task-submission";

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
    const state = submissionState();
    const restoreDraft = vi.fn<(request: unknown) => Promise<void>>().mockResolvedValue(undefined);
    const submission = createConversationTaskSubmission({
      input: {
        extensions: ref({ pluginIds: [], skillIds: [], connections: [] }),
        spaceId: ref("space"),
        selectedConversationId: ref("conversation"),
        conversation: ref(),
        prompt: ref("prompt"),
        target: ref({ type: "agent", runtimeKind: "codex" }),
        attachments: ref([]),
        visibleAttachments: ref([]),
        hasUnresolvedFileReferences: ref(false),
        messages: shallowRef([]),
        clearDraft: vi.fn<(request: unknown) => Promise<void>>().mockResolvedValue(undefined),
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

  it("keeps and reports the draft when persisted deletion rejects", async () => {
    const state = submissionState();
    const restoreDraft = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);
    const clearDraft = vi
      .fn<() => Promise<void>>()
      .mockRejectedValue(new Error("draft database is read-only"));
    const prompt = ref("unsent work");
    const submission = createConversationTaskSubmission({
      input: {
        extensions: ref({ pluginIds: [], skillIds: [], connections: [] }),
        spaceId: ref("space"),
        selectedConversationId: ref("conversation"),
        conversation: ref(),
        prompt,
        target: ref({ type: "agent", runtimeKind: "codex" }),
        attachments: ref([]),
        visibleAttachments: ref([]),
        hasUnresolvedFileReferences: ref(false),
        messages: shallowRef([]),
        clearDraft,
        restoreDraft,
      },
      state,
      accepted: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
      settled: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
      restoreForSpace: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
    });

    await submission.send();

    expect(ipc.startConversationTask).not.toHaveBeenCalled();
    expect(restoreDraft).toHaveBeenCalledOnce();
    expect(state.error.value).toContain("draft database is read-only");
    expect(state.loading.value).toBe(false);
    expect(prompt.value).toBe("unsent work");
  });
});

function submissionState() {
  return {
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
}
