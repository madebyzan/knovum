import { createPinia, setActivePinia } from "pinia";
import { describe, expect, it, vi } from "vitest";
import { reactive } from "vue";

import { promptArguments, resourceParameters } from "../../../../foundation/extensions/catalog-input";
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

describe("frozen extension input", () => {
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
    setActivePinia(createPinia());
    ipc.saveConversationDraft.mockImplementation(async (request) => request);
    const drafts = useConversationDraftStore();
    await drafts.save("space", "conversation", {
      prompt: "Prompt",
      attachments: [],
      extensions: reactive(original),
      reacquireFileNames: [],
    });
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
  it("validates actual RFC6570 parameter kinds and exact prompt argument values", () => {
    expect(resourceParameters('{"id":"a/b","labels":["one"],"pairs":{"key":"value"}}')).toEqual({
      id: "a/b",
      labels: ["one"],
      pairs: { key: "value" },
    });
    expect(() => resourceParameters('{"id":12}')).toThrow(/Invalid input/);
    expect(() => promptArguments('{"subject":12}')).toThrow(/expected string/);
  });
});
