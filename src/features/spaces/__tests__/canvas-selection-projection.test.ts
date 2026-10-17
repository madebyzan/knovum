import { describe, expect, it } from "vitest";

import { selectedCanvasNodesInOrder } from "../canvas/editor-session";
import { createSpaceNode } from "../canvas/node";
import { creationSources } from "../knowledge-derivations/catalog/creation-source";

describe("canvas selection projection", () => {
  it("preserves user selection order when pinning creation sources", () => {
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
    expect(creationSources(selected)).toEqual([
      { nodeId: "second", title: "Second", sourceRevision: "r2", revisionBasis: "knowledge" },
      { nodeId: "first", title: "First", sourceRevision: "r1", revisionBasis: "knowledge" },
    ]);
  });

  it("pins an image on its pixels, with or without analysis", () => {
    const image = createSpaceNode(
      {
        kind: "image",
        title: "Shot",
        fileName: "shot.png",
        assetPath: "/managed/shot.png",
        sourceRevision: "asset:1",
        knowledgeRevision: "node1:abc",
      },
      { x: 0, y: 0 },
      "image",
    );
    expect(creationSources([image])).toEqual([
      { nodeId: "image", title: "Shot", sourceRevision: "asset:1", revisionBasis: "asset" },
    ]);
  });

  it("leaves out a resource the Run could not send", () => {
    const processing = createSpaceNode(
      {
        kind: "image",
        title: "Pending",
        fileName: "pending.png",
        assetPath: "/managed/pending.png",
        sourceRevision: "asset:1",
        processingState: "processing",
      },
      { x: 0, y: 0 },
      "pending",
    );
    expect(creationSources([processing])).toEqual([]);
  });
});
