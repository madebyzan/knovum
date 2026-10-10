import { describe, expect, it, vi } from "vitest";
import { ref } from "vue";

import { deferred } from "../../__tests__/support/canvas-fixtures";
import type { ConversationDraft } from "../../contracts/generated/ConversationDraft";
import { createConversationDraftSession } from "../conversation-draft-session";

type ComposerDraft = Omit<ConversationDraft, "spaceId" | "conversationId">;

function draft(prompt: string): ComposerDraft {
  return {
    prompt,
    attachments: [],
    extensions: { pluginIds: [], skillIds: [prompt], connections: [] },
    reacquireFileNames: [],
  };
}

describe("conversation draft session", () => {
  it("rejects a stale A restore after A-B-A navigation and remembers the admitted owner snapshot", async () => {
    const firstA = deferred<ComposerDraft | undefined>();
    const spaceB = deferred<ComposerDraft | undefined>();
    const secondA = deferred<ComposerDraft | undefined>();
    const save = vi
      .fn<(spaceId: string, conversationId: string, draft: ComposerDraft) => Promise<void>>()
      .mockResolvedValue(undefined);
    const drafts = {
      read: vi.fn<(spaceId: string, conversationId: string) => ComposerDraft | undefined>(),
      load: vi
        .fn<(spaceId: string, conversationId: string) => Promise<ComposerDraft | undefined>>()
        .mockReturnValueOnce(firstA.promise)
        .mockReturnValueOnce(spaceB.promise)
        .mockReturnValueOnce(secondA.promise),
      save,
      move: vi
        .fn<(spaceId: string, fromConversationId: string, toConversationId: string) => Promise<void>>()
        .mockResolvedValue(undefined),
      remove: vi
        .fn<(spaceId: string, conversationId: string) => Promise<void>>()
        .mockResolvedValue(undefined),
    };
    const spaceId = ref<string | undefined>("space-a");
    const selectedConversationId = ref("conversation");
    const prompt = ref("");
    const attachments = ref<ComposerDraft["attachments"]>([]);
    const extensions = ref({ pluginIds: [], skillIds: [] as string[], connections: [] });
    const reacquireFileNames = ref<string[]>([]);
    const session = createConversationDraftSession({
      drafts,
      spaceId,
      selectedConversationId,
      prompt,
      attachments,
      extensions,
      reacquireFileNames,
    });

    const restoringFirstA = session.restore("conversation", "space-a");
    spaceId.value = "space-b";
    const restoringB = session.restore("conversation", "space-b");
    spaceId.value = "space-a";
    const restoringSecondA = session.restore("conversation", "space-a");

    secondA.resolve(draft("newest A"));
    await expect(restoringSecondA).resolves.toBe(true);
    expect(prompt.value).toBe("newest A");

    firstA.resolve(draft("stale A"));
    spaceB.resolve(draft("stale B"));
    await expect(restoringFirstA).resolves.toBe(false);
    await expect(restoringB).resolves.toBe(false);
    expect(prompt.value).toBe("newest A");

    const remembering = session.remember();
    spaceId.value = "space-b";
    prompt.value = "mutated after save";
    extensions.value.skillIds.push("late mutation");
    await remembering;
    expect(save).toHaveBeenCalledWith("space-a", "conversation", {
      prompt: "newest A",
      attachments: [],
      extensions: { pluginIds: [], skillIds: ["newest A"], connections: [] },
      reacquireFileNames: [],
    });
  });
});
