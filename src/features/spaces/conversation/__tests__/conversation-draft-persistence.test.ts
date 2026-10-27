import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { effectScope, ref } from "vue";

import { deferred } from "../../../../shared/__tests__/support/deferred";
import type { AgentExtensionSelection } from "../../../../shared/contracts/generated/AgentExtensionSelection";
import type { ConversationAttachment } from "../../contracts/generated/ConversationAttachment";
import type { ConversationDraft } from "../../contracts/generated/ConversationDraft";

type ComposerDraft = Omit<ConversationDraft, "spaceId" | "conversationId">;

const drafts = vi.hoisted(() => ({
  read: vi.fn<(spaceId: string, conversationId: string) => ComposerDraft | undefined>(),
  load: vi.fn<(spaceId: string, conversationId: string) => Promise<ComposerDraft | undefined>>(),
  save: vi.fn<(spaceId: string, conversationId: string, draft: ComposerDraft) => Promise<void>>(),
  move: vi.fn<(spaceId: string, fromConversationId: string, toConversationId: string) => Promise<void>>(),
  remove: vi.fn<(spaceId: string, conversationId: string) => Promise<void>>(),
}));

vi.mock("../conversation-draft-store", () => ({ useConversationDraftStore: () => drafts }));

import { createConversationFilePicker } from "../conversation-file-reacquisition";
import { useConversationDraftPersistence } from "../use-conversation-draft-persistence";

function draft(reacquireFileNames: string[]): ComposerDraft {
  return {
    prompt: "preserved prompt",
    attachments: [],
    extensions: { pluginIds: [], skillIds: [], connections: [] },
    reacquireFileNames,
  };
}

function setup() {
  const scope = effectScope();
  const spaceId = ref<string | undefined>("space");
  const conversationId = ref("conversation");
  const prompt = ref("");
  const attachments = ref<ConversationAttachment[]>([]);
  const extensions = ref<AgentExtensionSelection>({ pluginIds: [], skillIds: [], connections: [] });
  const persistence = scope.run(() =>
    useConversationDraftPersistence({ spaceId, conversationId, prompt, attachments, extensions }),
  );
  if (!persistence) throw new Error("draft persistence scope did not initialize");
  return { scope, spaceId, conversationId, prompt, attachments, persistence };
}

describe("conversation draft persistence", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.clearAllMocks();
    drafts.save.mockResolvedValue(undefined);
    drafts.move.mockResolvedValue(undefined);
    drafts.remove.mockResolvedValue(undefined);
  });
  afterEach(() => vi.useRealTimers());

  it("persists a single discarded recovery reference without clearing the draft", async () => {
    drafts.load.mockResolvedValue(draft(["discard.pdf", "keep.pdf"]));
    const state = setup();
    await state.persistence.session.restore("conversation");
    await vi.advanceTimersByTimeAsync(300);
    drafts.save.mockClear();

    state.persistence.session.discardFileReference("discard.pdf");
    await vi.advanceTimersByTimeAsync(300);

    expect(drafts.save).toHaveBeenCalledWith("space", "conversation", {
      prompt: "preserved prompt",
      attachments: [],
      extensions: { pluginIds: [], skillIds: [], connections: [] },
      reacquireFileNames: ["keep.pdf"],
    });
    state.scope.stop();
  });

  it("invalidates pending file acquisition when its owning scope is disposed", async () => {
    drafts.load.mockResolvedValue(draft(["lost.pdf"]));
    const state = setup();
    await state.persistence.session.restore("conversation");
    const selected = deferred<void>();
    const pick = createConversationFilePicker({
      spaceId: state.spaceId,
      conversationId: state.conversationId,
      ownerEpoch: state.persistence.session.ownerEpoch,
      attachments: state.attachments,
      reacquireFileNames: state.persistence.reacquireFileNames,
      choose: async (valid) => {
        await selected.promise;
        if (valid())
          state.attachments.value = [{ kind: "file", path: "/picked/lost.pdf", sourceRevision: "rev" }];
      },
    });

    const pending = pick();
    state.scope.stop();
    selected.resolve();
    await pending;

    expect(state.attachments.value).toEqual([]);
    expect(state.persistence.reacquireFileNames.value).toEqual(["lost.pdf"]);
  });

  it("never shows a failed save of a draft the composer has already left", async () => {
    drafts.load.mockResolvedValue(undefined);
    const state = setup();
    await state.persistence.session.restore("conversation");
    const rejected = deferred<void>();
    drafts.save.mockReturnValueOnce(rejected.promise);
    state.prompt.value = "typed into a Conversation being deleted";
    await vi.advanceTimersByTimeAsync(300);

    state.conversationId.value = "";
    await state.persistence.session.restore("");
    rejected.reject(new Error("conversation-draft-unavailable"));
    await vi.advanceTimersByTimeAsync(0);

    expect(state.persistence.error.value).toBeUndefined();
    state.scope.stop();
  });

  it("clears a draft error once the composer shows another Conversation", async () => {
    drafts.load.mockResolvedValue(undefined);
    const state = setup();
    await state.persistence.session.restore("conversation");
    drafts.save.mockRejectedValueOnce(new Error("conversation-draft-invalid"));
    state.prompt.value = "too large";
    await vi.advanceTimersByTimeAsync(300);
    expect(state.persistence.error.value).toBe("conversation-draft-invalid");

    state.conversationId.value = "";
    expect(state.persistence.error.value).toBeUndefined();
    state.scope.stop();
  });
});
