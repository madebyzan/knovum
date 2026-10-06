import { createPinia, setActivePinia } from "pinia";
import { describe, expect, it } from "vitest";
import { reactive } from "vue";

import { promptArguments, resourceParameters } from "../../../foundation/extensions/catalog-input";
import { useConversationDraftStore } from "../conversation/conversation-draft-store";
describe("frozen extension input", () => {
  it("copies nested resource parameters so editing a draft cannot change a submitted selection", () => {
    const original = {
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
    const drafts = useConversationDraftStore();
    drafts.save("space", "conversation", {
      prompt: "Prompt",
      attachments: [],
      extensions: reactive(original),
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
