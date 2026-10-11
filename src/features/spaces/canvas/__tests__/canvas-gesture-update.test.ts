import { describe, expect, it } from "vitest";

import {
  applyCanvasGestureUpdates,
  captureCanvasGestureLayouts,
} from "../konva/document/canvas-document-mutations";
import type { CanvasDocumentV1 } from "../schema/document";

const document: CanvasDocumentV1 = {
  version: 1,
  viewport: { x: 0, y: 0, zoom: 1 },
  nodes: [
    {
      nodeId: "text",
      frame: { x: 10, y: 20, width: 240, height: 40, rotation: 0 },
      sizingMode: "frame",
      zIndex: 0,
    },
  ],
};

describe("canvas gesture layout updates", () => {
  it("commits frame and typography scale as one layout mutation", () => {
    const next = applyCanvasGestureUpdates(document, [
      {
        nodeId: "text",
        frame: { x: 10, y: 20, width: 480, height: 80, rotation: 0 },
        appearance: { typographyScale: 2 },
      },
    ]);
    expect(next.nodes[0]).toMatchObject({
      frame: { width: 480, height: 80 },
      appearance: { typographyScale: 2 },
    });
  });

  it("captures absence explicitly so undo removes gesture-created appearance", () => {
    const captured = captureCanvasGestureLayouts(document, [
      { nodeId: "text", frame: { ...document.nodes[0]!.frame } },
    ]);
    const scaled = applyCanvasGestureUpdates(document, [
      { ...captured[0]!, appearance: { typographyScale: 2 } },
    ]);
    expect(applyCanvasGestureUpdates(scaled, captured).nodes[0]?.appearance).toBeUndefined();
  });
});
