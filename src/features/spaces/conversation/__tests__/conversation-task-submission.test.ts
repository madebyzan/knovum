import { beforeEach, describe, expect, it, vi } from "vitest";
import { ref, shallowRef, type Ref } from "vue";

type TaskUpdate =
  | { type: "content_delta"; task_id: string; text: string }
  | { type: "content_boundary" }
  | { type: "tool_started"; tool_call_id: string };

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
const admission = vi.hoisted(() => ({
  admitAiRequirements: vi.fn<(requirements: unknown) => Promise<boolean>>(),
}));

vi.mock("../conversation-ipc", () => ipc);
vi.mock("../conversation-run-observation", () => observation);
vi.mock("../../../../foundation/ai/ai-readiness-admission", () => admission);

import { createConversationTaskSubmission } from "../conversation-task-submission";
import type { LiveTranscript } from "../conversation-transcript";

const accepted = { taskId: "task", runId: "run", conversationId: "conversation", branchId: "branch" };

describe("conversation task settlement", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    admission.admitAiRequirements.mockResolvedValue(true);
  });

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

  it("keeps the live transcript on screen until the stored answer and the ledger have been loaded", async () => {
    ipc.startConversationTask.mockImplementation(async (_request, onUpdate) => {
      onUpdate({ type: "content_delta", task_id: "task", text: "**Done**" });
      return { accepted, settled: Promise.resolve({ type: "finished", state: "succeeded" }) };
    });
    const shownWhileLoading: unknown[] = [];
    const { state, submission } = createSubmission({
      settled: vi.fn<() => Promise<void>>(async () => {
        shownWhileLoading.push(state.live.value);
      }),
      loadActivity: vi.fn<() => Promise<void>>(async () => {
        shownWhileLoading.push(state.live.value);
      }),
    });

    await submission.send();

    expect(shownWhileLoading).toEqual([
      [{ kind: "text", text: "**Done**" }],
      [{ kind: "text", text: "**Done**" }],
    ]);
    expect(state.live.value).toBeUndefined();
  });

  it("records where the run called a tool between the words it wrote", async () => {
    // The tools a run calls belong between the words that led to them and the words that follow;
    // a turn that lists every call first moved its text down each time another call began.
    let shown: unknown;
    ipc.startConversationTask.mockImplementation(async (_request, onUpdate) => {
      onUpdate({ type: "content_delta", task_id: "task", text: "Looking at the Space." });
      onUpdate({ type: "tool_started", tool_call_id: "search" });
      onUpdate({ type: "tool_started", tool_call_id: "search" });
      onUpdate({ type: "content_boundary" });
      onUpdate({ type: "content_delta", task_id: "task", text: "Three notes " });
      onUpdate({ type: "content_delta", task_id: "task", text: "disagree." });
      onUpdate({ type: "content_boundary" });
      onUpdate({ type: "content_delta", task_id: "task", text: "Next message." });
      return { accepted, settled: Promise.resolve({ type: "finished", state: "succeeded" }) };
    });
    const { state, submission } = createSubmission({
      settled: vi.fn<() => Promise<void>>(async () => {
        shown = state.live.value;
      }),
    });

    await submission.send();

    expect(shown).toEqual([
      { kind: "text", text: "Looking at the Space." },
      { kind: "tool", id: "search" },
      { kind: "text", text: "Three notes disagree." },
      { kind: "text", text: "Next message." },
    ]);
  });

  it("creates an unsaved Space before anything is written for the conversation", async () => {
    const order: string[] = [];
    ipc.startConversationTask.mockImplementation(async () => {
      order.push("start");
      return { accepted, settled: Promise.resolve({ type: "finished", state: "succeeded" }) };
    });
    const { submission } = createSubmission({
      ensureSpacePersisted: vi.fn<() => Promise<void>>(async () => {
        order.push("persist Space");
      }),
      clearDraft: vi.fn<() => Promise<void>>(async () => {
        order.push("clear draft");
      }),
    });

    await submission.send();

    expect(order).toEqual(["persist Space", "clear draft", "start"]);
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
  ensureSpacePersisted?: () => Promise<void>;
  clearDraft?: () => Promise<void>;
  settled?: () => Promise<void>;
  loadActivity?: () => Promise<void>;
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
      ensureSpacePersisted:
        overrides.ensureSpacePersisted ?? vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
      clearDraft: overrides.clearDraft ?? vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
      restoreDraft,
    },
    state,
    accepted: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
    settled: overrides.settled ?? vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
    loadActivity: overrides.loadActivity ?? vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
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
    live: shallowRef<LiveTranscript>(),
    activityRevision: shallowRef(0),
  };
}
