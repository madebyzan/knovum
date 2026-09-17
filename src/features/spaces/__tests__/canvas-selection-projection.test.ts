import { describe, expect, it } from "vitest";

import { selectedCanvasNodesInOrder } from "../canvas/editor-session";
import { createSpaceNode } from "../canvas/node";
import { groundedDocumentSources } from "../knowledge-derivations/catalog/grounded-document";

describe("canvas selection projection", () => {
  it("preserves user selection order when choosing the primary derivation source", () => {
    const first = createSpaceNode(
      { kind: "markdown", title: "First", text: "First", knowledgeRevision: "r1" },
      { x: 0, y: 0 },
      "first",
    );
    const second = createSpaceNode(
      { kind: "markdown", title: "Second", text: "Second", knowledgeRevision: "r2" },
      { x: 0, y: 0 },
      "second",
    );

    const selected = selectedCanvasNodesInOrder([first, second], ["second", "first"]);
    expect(selected.map(({ id }) => id)).toEqual(["second", "first"]);
    expect(groundedDocumentSources(selected)?.map(({ nodeId, role }) => [nodeId, role])).toEqual([
      ["second", "primary"],
      ["first", "source"],
    ]);
  });
});
