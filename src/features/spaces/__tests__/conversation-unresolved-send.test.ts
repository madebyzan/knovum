import { describe, expect, it, vi } from "vitest";
import { ref, shallowRef } from "vue";

import type { startConversationTask } from "../conversation/conversation-ipc";

const ipc = vi.hoisted(() => ({ startConversationTask: vi.fn<typeof startConversationTask>() }));
vi.mock("../conversation/conversation-ipc", () => ipc);

import { createConversationTaskSubmission } from "../conversation/conversation-task-submission";

describe("conversation unresolved file admission", () => {
  it("does not clear or submit the draft through the programmatic send path", async () => {
    const clearDraft = vi.fn<() => Promise<void>>().mockResolvedValue(undefined);
    const messages = shallowRef([]);
    const submission = createConversationTaskSubmission({
      input: {
        extensions: ref({ pluginIds: [], skillIds: [], connections: [] }),
        spaceId: ref("space"),
        selectedConversationId: ref("conversation"),
        conversation: ref(),
        prompt: ref("keep this draft"),
        target: ref({ type: "agent", runtimeKind: "codex" }),
        attachments: ref([]),
        visibleAttachments: ref([]),
        hasUnresolvedFileReferences: ref(true),
        messages,
        clearDraft,
        restoreDraft: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
      },
      state: {
        activeRun: shallowRef(),
        activeRunId: shallowRef<string>(),
        activeConversationId: shallowRef(""),
        activeSpaceId: shallowRef<string>(),
        taskState: shallowRef<string>(),
        loading: shallowRef(false),
        error: shallowRef<string>(),
        streamedText: shallowRef(""),
        activityRevision: shallowRef(0),
      },
      accepted: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
      settled: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
      restoreForSpace: vi.fn<() => Promise<void>>().mockResolvedValue(undefined),
    });

    await submission.send();

    expect(clearDraft).not.toHaveBeenCalled();
    expect(ipc.startConversationTask).not.toHaveBeenCalled();
    expect(messages.value).toEqual([]);
  });
});
