import { describe, expect, it } from "vitest";
import { reactive } from "vue";

import { promptArguments, resourceParameters } from "../../../foundation/extensions/catalog-input";
import { copyExtensionSelection } from "../../../foundation/extensions/extension-selection";
describe("frozen extension input", () => {
  it("copies nested resource parameters so editing a draft cannot change a submitted selection", () => {
    const original = {
      skillIds: ["skill"],
      connectionIds: ["connection"],
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
    const frozen = copyExtensionSelection(reactive(original));
    original.connections[0]!.resourceTemplates[0]!.parameters.labels.push("changed");
    expect(frozen.connections[0]!.resourceTemplates[0]!.parameters).toEqual({
      id: "one",
      labels: ["reviewed"],
    });
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
