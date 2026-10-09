import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { deferred } from "../../__tests__/support/canvas-fixtures";
import type { ConversationDraft } from "../../contracts/generated/ConversationDraft";
import type { SaveConversationDraftRequest } from "../../contracts/generated/SaveConversationDraftRequest";

const ipc = vi.hoisted(() => ({
  loadConversationDraft:
    vi.fn<(spaceId: string, conversationId: string) => Promise<ConversationDraft | null>>(),
  moveConversationDraft:
    vi.fn<(spaceId: string, fromConversationId: string, toConversationId: string) => Promise<void>>(),
  removeConversationDraft: vi.fn<(spaceId: string, conversationId: string) => Promise<void>>(),
  saveConversationDraft: vi.fn<(request: SaveConversationDraftRequest) => Promise<ConversationDraft>>(),
}));

vi.mock("../conversation-ipc", () => ipc);

import { useConversationDraftStore } from "../conversation-draft-store";

type ComposerDraft = Omit<ConversationDraft, "spaceId" | "conversationId">;

function persisted(prompt: string, reacquireFileNames: string[]): ConversationDraft {
  return {
    spaceId: "space",
    conversationId: "conversation",
    prompt,
    attachments: [],
    extensions: { skillIds: [], connections: [] },
    reacquireFileNames,
  };
}

function composer(prompt: string): ComposerDraft {
  return {
    prompt,
    attachments: [],
    extensions: { skillIds: [], connections: [] },
    reacquireFileNames: [],
  };
}

describe("conversation draft persistence ordering", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    setActivePinia(createPinia());
  });

  it("serializes writes and prevents an older response from replacing the newer cache", async () => {
    const firstWrite = deferred<ConversationDraft>();
    const secondWrite = deferred<ConversationDraft>();
    ipc.saveConversationDraft
      .mockReturnValueOnce(firstWrite.promise)
      .mockReturnValueOnce(secondWrite.promise);
    const store = useConversationDraftStore();

    const first = store.save("space", "conversation", composer("first"));
    const second = store.save("space", "conversation", composer("second"));
    await Promise.resolve();
    expect(ipc.saveConversationDraft).toHaveBeenCalledTimes(1);

    firstWrite.resolve(persisted("first", ["old-external.txt"]));
    await first;
    expect(ipc.saveConversationDraft).toHaveBeenCalledTimes(2);
    expect(store.read("space", "conversation")).toMatchObject({
      prompt: "second",
      reacquireFileNames: [],
    });

    secondWrite.resolve(persisted("second", ["new-external.txt"]));
    await second;
    expect(store.read("space", "conversation")).toMatchObject({
      prompt: "second",
      reacquireFileNames: ["new-external.txt"],
    });
  });
});
