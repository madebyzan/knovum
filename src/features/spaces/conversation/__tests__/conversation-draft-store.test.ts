import { createPinia, setActivePinia } from "pinia";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { reactive } from "vue";

import { deferred } from "../../../../shared/__tests__/support/deferred";
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
    extensions: { pluginIds: [], skillIds: [], connections: [] },
    reacquireFileNames,
  };
}

function composer(prompt: string): ComposerDraft {
  return {
    prompt,
    attachments: [],
    extensions: { pluginIds: [], skillIds: [], connections: [] },
    reacquireFileNames: [],
  };
}

describe("conversation draft persistence", () => {
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
  it("copies nested resource parameters so editing a draft cannot change a submitted selection", async () => {
    const original = {
      pluginIds: [],
      skillIds: ["skill"],
      connections: [
        {
          connectionId: "connection",
          toolNames: ["read"],
          resourceUris: [],
          resourceTemplates: [
            { uriTemplate: "resource://{id}{?labels*}", parameters: { id: "one", labels: ["reviewed"] } },
          ],
          promptNames: [],
        },
      ],
    };
    ipc.saveConversationDraft.mockImplementation(async (request) => request);
    const drafts = useConversationDraftStore();
    await drafts.save("space", "conversation", { ...composer("Prompt"), extensions: reactive(original) });
    const frozen = drafts.read("space", "conversation")!.extensions;
    expect(frozen).toEqual(original);
    frozen.skillIds.push("another-skill");
    frozen.connections[0]!.toolNames.push("another-tool");
    expect(original.skillIds).toEqual(["skill"]);
    expect(original.connections[0]!.toolNames).toEqual(["read"]);
    original.connections[0]!.resourceTemplates[0]!.parameters.labels.push("changed");
    expect(frozen.connections[0]!.resourceTemplates[0]!.parameters).toEqual({
      id: "one",
      labels: ["reviewed"],
    });
    expect(drafts.read("space", "conversation")!.extensions.skillIds).toEqual(["skill"]);
    expect(drafts.read("space", "conversation")!.extensions.connections[0]!.toolNames).toEqual(["read"]);
    expect(
      ipc.saveConversationDraft.mock.calls[0]![0].extensions.connections[0]!.resourceTemplates[0]!.parameters,
    ).toEqual({ id: "one", labels: ["reviewed"] });
  });
});
