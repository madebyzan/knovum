import { describe, expect, it } from "vitest";

import type { CanvasJsonImportRequest } from "../json-canvas-import-contract";
import { prepareJsonCanvasNodes } from "../json-canvas-node-preparation";

const frame = { x: 10, y: 20, width: 320, height: 180, rotation: 0 } as const;

describe("JSON Canvas domain node preparation", () => {
  it("maps standard content into domain nodes and preserves group membership", async () => {
    const preparation = await prepareJsonCanvasNodes(
      request([
        {
          sourceId: "group",
          frame,
          zIndex: 0,
          appearance: { fill: "blue" },
          content: { type: "group", label: "Sources" },
        },
        {
          sourceId: "text",
          parentSourceId: "group",
          frame,
          zIndex: 1,
          content: { type: "text", text: "Evidence" },
        },
        {
          sourceId: "link",
          parentSourceId: "group",
          frame,
          zIndex: 2,
          content: { type: "link", url: "https://example.com/research" },
        },
      ]),
    );

    const [group, text, link] = preparation.nodes.map(({ node }) => node);
    expect(group?.data).toEqual({ kind: "section", title: "Sources" });
    expect(text?.data).toMatchObject({ kind: "note", plainText: "Evidence", sectionId: group?.id });
    expect(link?.data).toMatchObject({
      kind: "website",
      url: "https://example.com/research",
      sectionId: group?.id,
    });
  });
});

function request(nodes: CanvasJsonImportRequest["plan"]["nodes"]): CanvasJsonImportRequest {
  return {
    sourceDirectory: "/unused",
    plan: { nodes, edges: [] },
    baseRevision: 0,
    document: { version: 1, viewport: { x: 0, y: 0, zoom: 1 }, nodes: [] },
  };
}
