import { describe, expect, it } from "vitest";

import { exportJsonCanvasDocument } from "../json-canvas-mapping";

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
});
