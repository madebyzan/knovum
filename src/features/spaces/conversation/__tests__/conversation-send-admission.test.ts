import { beforeEach, describe, expect, it, vi } from "vitest";
import { ref, shallowRef } from "vue";

import type { admitAiRequirements } from "../../../../foundation/ai/ai-readiness-admission";
import type { startConversationTask } from "../conversation-ipc";

const ipc = vi.hoisted(() => ({ startConversationTask: vi.fn<typeof startConversationTask>() }));
const admission = vi.hoisted(() => ({ admitAiRequirements: vi.fn<typeof admitAiRequirements>() }));
vi.mock("../conversation-ipc", () => ipc);
vi.mock("../../../../foundation/ai/ai-readiness-admission", () => admission);

import { createConversationTaskSubmission } from "../conversation-task-submission";
import type { LiveTranscript } from "../conversation-transcript";

const codex = { type: "agent", runtimeKind: "codex" } as const;

function submissionFor(options: { hasUnresolvedFileReferences: boolean }) {
  const clearDraft = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);
  const prompt = ref("keep this draft");
  const messages = shallowRef([]);
  const loading = shallowRef(false);
  const error = shallowRef<string>();
  const submission = createConversationTaskSubmission({
    input: {
      extensions: ref({ pluginIds: [], skillIds: [], connections: [] }),
      spaceId: ref("space"),
      selectedConversationId: ref("conversation"),
      conversation: ref(),
      prompt,
      target: ref(codex),
      attachments: ref([]),
      visibleAttachments: ref([]),
      hasUnresolvedFileReferences: ref(options.hasUnresolvedFileReferences),
      messages,
      ensureSpacePersisted: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
      clearDraft,
      restoreDraft: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
    },
    state: {
      activeRun: shallowRef(),
      activeRunId: shallowRef<string>(),
      activeConversationId: shallowRef(""),
      activeSpaceId: shallowRef<string>(),
      taskState: shallowRef<string>(),
      loading,
      error,
      live: shallowRef<LiveTranscript>(),
      activityRevision: shallowRef(0),
    },
    accepted: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
    settled: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
    loadActivity: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
    restoreForSpace: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
  });
  return { submission, clearDraft, prompt, messages, loading, error };
}

describe("conversation send admission", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    admission.admitAiRequirements.mockResolvedValue(true);
  });

  it("does not clear or submit the draft while file references are unresolved", async () => {
    const { submission, clearDraft, messages } = submissionFor({ hasUnresolvedFileReferences: true });

    await submission.send();

    expect(clearDraft).not.toHaveBeenCalled();
    expect(ipc.startConversationTask).not.toHaveBeenCalled();
    expect(messages.value).toEqual([]);
  });

  it("leaves the draft in place when the chosen executor cannot start", async () => {
    admission.admitAiRequirements.mockResolvedValue(false);
    const { submission, clearDraft, prompt, messages, loading, error } = submissionFor({
      hasUnresolvedFileReferences: false,
    });

    await submission.send();

    expect(admission.admitAiRequirements).toHaveBeenCalledWith([
      { kind: "routed", purpose: "knowledge", target: codex },
    ]);
    expect(clearDraft).not.toHaveBeenCalled();
    expect(ipc.startConversationTask).not.toHaveBeenCalled();
    expect(messages.value).toEqual([]);
    expect(loading.value).toBe(false);
    expect(error.value).toBeUndefined();
    expect(prompt.value).toBe("keep this draft");
  });
});
