import { describe, expect, it } from "vitest";

import { createSpaceNode } from "../canvas/node";
import { knowledgeVideoInputs } from "../knowledge-derivations/video/knowledge-video";

const documentNode = () =>
  createSpaceNode(
    {
      kind: "markdown",
      title: "Knowledge script",
      text: "Use Image 1 as the visual reference.",
      knowledgeRevision: "node1:script",
    },
    { x: 0, y: 0 },
    "script",
  );

const imageNode = () =>
  createSpaceNode(
    {
      kind: "image",
      title: "Reference",
      fileName: "reference.png",
      assetPath: "/spaces/assets/reference.png",
      sourceRevision: "sha256:image",
    },
    { x: 100, y: 0 },
    "image",
  );

describe("knowledge video inputs", () => {
  it("keeps current Markdown first and selected images in canvas order", () => {
    const inputs = knowledgeVideoInputs([documentNode(), imageNode()]);
    expect(inputs?.document.id).toBe("script");
    expect(inputs?.images.map(({ id }) => id)).toEqual(["image"]);
  });

  it("rejects Markdown without a knowledge revision and unrelated selected resources", () => {
    const draft = createSpaceNode(
      { kind: "markdown", title: "Draft", text: "Draft" },
      { x: 0, y: 0 },
      "draft",
    );
    const note = createSpaceNode(
      { kind: "note", title: "Note", plainText: "text", tiptapContent: { type: "doc", content: [] } },
      { x: 0, y: 0 },
      "note",
    );
    expect(knowledgeVideoInputs([draft, imageNode()])).toBeUndefined();
    expect(knowledgeVideoInputs([documentNode(), note])).toBeUndefined();
  });
});
