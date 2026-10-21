import { beforeEach, describe, expect, it, vi } from "vitest";
import { ref, shallowRef, type Ref } from "vue";

type TaskUpdate = { type: "content_delta"; text: string };

const ipc = vi.hoisted(() => ({
  getConversationTaskError: vi.fn<(runId: string) => Promise<string | null>>(),
  startConversationTask: vi.fn<
    (
      request: unknown,
      onUpdate: (update: TaskUpdate) => void,
    ) => Promise<{
      accepted: { taskId: string; runId: string; conversationId: string; branchId: string };
      settled: Promise<unknown>;
    }>
  >(),
}));
const observation = vi.hoisted(() => ({ observeConversationRun: vi.fn<() => void>() }));

vi.mock("../conversation-ipc", () => ipc);
vi.mock("../conversation-run-observation", () => observation);

import { createConversationTaskSubmission } from "../conversation-task-submission";

const accepted = { taskId: "task", runId: "run", conversationId: "conversation", branchId: "branch" };

describe("conversation task settlement", () => {
  beforeEach(() => vi.clearAllMocks());

  it("shows a channel settlement failure without treating it as a durable run state", async () => {
    ipc.startConversationTask.mockResolvedValue({
      accepted,
      settled: Promise.resolve({
        type: "finished",
        state: "settlement_failed",
        error: "final state could not be read",
      }),
    });
    const { state, restoreDraft, submission } = createSubmission({
      settled: vi.fn<() => Promise<void>>().mockRejectedValue(new Error("projection database unavailable")),
    });

    await submission.send();

    expect(state.taskState.value).toBe("settlement_failed");
    expect(state.error.value).toContain("final state could not be read");
    expect(state.loading.value).toBe(false);
    expect(state.activeRunId.value).toBeUndefined();
    expect(ipc.getConversationTaskError).not.toHaveBeenCalled();
    expect(restoreDraft).not.toHaveBeenCalled();
  });

  it("keeps the streamed answer on screen until the stored answer has been loaded", async () => {
    ipc.startConversationTask.mockImplementation(async (_request, onUpdate) => {
      onUpdate({ type: "content_delta", text: "**Done**" });
      return { accepted, settled: Promise.resolve({ type: "finished", state: "succeeded" }) };
    });
    const shownWhileLoading: string[] = [];
    const { state, submission } = createSubmission({
      settled: vi.fn<() => Promise<void>>(async () => {
        shownWhileLoading.push(state.streamedText.value);
      }),
    });

    await submission.send();

    expect(shownWhileLoading).toEqual(["**Done**"]);
    expect(state.streamedText.value).toBe("");
  });

  it("keeps and reports the draft when persisted deletion rejects", async () => {
    const prompt = ref("unsent work");
    const { state, restoreDraft, submission } = createSubmission({
      prompt,
      clearDraft: vi.fn<() => Promise<void>>().mockRejectedValue(new Error("draft database is read-only")),
    });

    await submission.send();

    expect(ipc.startConversationTask).not.toHaveBeenCalled();
    expect(restoreDraft).toHaveBeenCalledOnce();
    expect(state.error.value).toContain("draft database is read-only");
    expect(state.loading.value).toBe(false);
    expect(prompt.value).toBe("unsent work");
  });
});

function createSubmission(overrides: {
  prompt?: Ref<string>;
  clearDraft?: () => Promise<void>;
  settled?: () => Promise<void>;
}) {
  const state = submissionState();
  const restoreDraft = vi.fn<(request: unknown) => Promise<void>>().mockResolvedValue(undefined);
  const submission = createConversationTaskSubmission({
    input: {
      extensions: ref({ pluginIds: [], skillIds: [], connections: [] }),
      spaceId: ref("space"),
      selectedConversationId: ref("conversation"),
      conversation: ref(),
      prompt: overrides.prompt ?? ref("prompt"),
      target: ref({ type: "agent", runtimeKind: "codex" }),
      attachments: ref([]),
      visibleAttachments: ref([]),
      hasUnresolvedFileReferences: ref(false),
      messages: shallowRef([]),
      clearDraft: overrides.clearDraft ?? vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
      restoreDraft,
    },
    state,
    accepted: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
    settled: overrides.settled ?? vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
    restoreForSpace: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
  });
  return { state, restoreDraft, submission };
}

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
