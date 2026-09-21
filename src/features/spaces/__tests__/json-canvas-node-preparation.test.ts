import { describe, expect, it } from "vitest";

import type { CanvasJsonImportRequest } from "../canvas-exchange/json-canvas-import-contract";
import { prepareJsonCanvasNodes } from "../session/actions/json-canvas-node-preparation";

describe("JSON Canvas content preparation", () => {
  it("creates content payloads and keeps Section membership in the separate import plan", async () => {
    const preparation = await prepareJsonCanvasNodes(request());

    expect(preparation.sectionIds.get("group")).toEqual(expect.any(String));
    expect(preparation.nodes.map(({ node }) => node.data)).toEqual([
      expect.objectContaining({ kind: "note", plainText: "Evidence" }),
      expect.objectContaining({ kind: "website", url: "https://example.com/research" }),
    ]);
    for (const { node } of preparation.nodes) {
      expect(node.data).not.toHaveProperty("sectionId");
      expect(node.data).not.toHaveProperty("locked");
    }
  });
});

function request(): CanvasJsonImportRequest {
  return {
    sourceDirectory: "/unused",
    baseRevision: 0,
    plan: {
      sections: [
        {
          sourceId: "group",
          frame: { x: 0, y: 0, width: 400, height: 300 },
          title: "Sources",
          order: 0,
        },
      ],
      contents: [
        {
          sourceId: "text",
          parentSectionSourceId: "group",
          frame: { x: 20, y: 50, width: 200, height: 120 },
          order: 1,
          content: { type: "text", text: "Evidence" },
        },
        {
          sourceId: "link",
          parentSectionSourceId: "group",
          frame: { x: 20, y: 180, width: 200, height: 100 },
          order: 2,
          content: { type: "link", url: "https://example.com/research" },
        },
      ],
      edges: [],
    },
  };
}
