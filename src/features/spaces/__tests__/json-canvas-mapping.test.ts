import { describe, expect, it } from "vitest";

import { exportJsonCanvasDocument, importJsonCanvasDocument } from "../canvas-exchange/json-canvas-mapping";

describe("JSON Canvas 1.0 mapping", () => {
  it("exports only standard fields and rounds geometry", () => {
    const document = exportJsonCanvasDocument(
      [
        {
          id: "note",
          frame: { x: 1.4, y: 2.6, width: 320, height: 180 },
          appearance: { fill: "pink" },
          content: { type: "text", text: "Evidence" },
        },
      ],
      [],
    );

    expect(document.nodes?.[0]).toEqual({
      id: "note",
      type: "text",
      text: "Evidence",
      x: 1,
      y: 3,
      width: 320,
      height: 180,
      color: "1",
    });
  });

  it("imports visual groups as Sections with separate membership", () => {
    const plan = importJsonCanvasDocument({
      nodes: [
        { id: "group", type: "group", label: "Sources", x: 0, y: 0, width: 400, height: 300 },
        { id: "note", type: "text", text: "Evidence", x: 20, y: 40, width: 200, height: 120 },
      ],
    });

    expect(plan.sections).toEqual([expect.objectContaining({ sourceId: "group", title: "Sources" })]);
    expect(plan.contents).toEqual([
      expect.objectContaining({
        sourceId: "note",
        parentSectionSourceId: "group",
        content: { type: "text", text: "Evidence" },
      }),
    ]);
  });

  it("rejects ambiguous visual group membership", () => {
    expect(() =>
      importJsonCanvasDocument({
        nodes: [
          { id: "left", type: "group", x: 0, y: 0, width: 400, height: 300 },
          { id: "right", type: "group", x: 0, y: 0, width: 400, height: 300 },
          { id: "note", type: "text", text: "Evidence", x: 20, y: 40, width: 200, height: 120 },
        ],
      }),
    ).toThrow(/ambiguous/u);
  });

  it("rejects relations whose endpoints are visual groups", () => {
    expect(() =>
      importJsonCanvasDocument({
        nodes: [
          { id: "group", type: "group", x: 0, y: 0, width: 400, height: 300 },
          { id: "note", type: "text", text: "Evidence", x: 20, y: 40, width: 200, height: 120 },
        ],
        edges: [{ id: "edge", fromNode: "group", toNode: "note" }],
      }),
    ).toThrow(/visual groups/u);
  });
});
