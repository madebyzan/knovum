import { describe, expect, it } from "vitest";

import type { CanvasDocumentV1 } from "../canvas/contract/canvas-document";
import { CanvasDocumentModel } from "../canvas/konva/document/canvas-document-model";
import { createNodeContentHeightBatch } from "../canvas/konva/node-content-height-batch";
import { createSpaceTextPayload } from "../editor/tiptap-content";

describe("node content height batch", () => {
  it("fits text height in both directions without creating history", async () => {
    let historyEntries = 0;
    const model = new CanvasDocumentModel(textDocument(), {
      documentChanged() {},
      historyEntry() {
        historyEntries += 1;
      },
      viewportChanged() {},
    });
    const contentHeight = createNodeContentHeightBatch(model, (id) =>
      id === "text"
        ? {
            id: "text",
            type: "text",
            data: createSpaceTextPayload("Text"),
            initialPosition: { x: 0, y: 0 },
          }
        : undefined,
    );

    contentHeight.fit("text", measurement(80));
    await Promise.resolve();
    expect(model.document.nodes[0]?.frame.height).toBe(80);
    contentHeight.fit("text", measurement(48));
    contentHeight.fit("text", measurement(32));
    await Promise.resolve();
    expect(model.document.nodes[0]?.frame.height).toBe(40);
    contentHeight.fit("text", measurement(160));
    await Promise.resolve();
    expect(model.document.nodes[0]?.frame.height).toBe(160);
    contentHeight.fit("text", measurement(720));
    await Promise.resolve();
    expect(model.document.nodes[0]?.frame.height).toBe(720);
    model.applyLocalDocument({
      ...model.document,
      nodes: model.document.nodes.map((layout) => ({
        ...layout,
        frame: { ...layout.frame, height: 10 },
        appearance: { typographyScale: 0.25 },
      })),
    });
    contentHeight.fit("text", { ...measurement(8), typographyScale: 0.25 });
    await Promise.resolve();
    expect(model.document.nodes[0]?.frame.height).toBe(10);
    expect(historyEntries).toBe(0);
  });

  it("defers content layout while a direct resize owns the frame", async () => {
    const model = new CanvasDocumentModel(textDocument(), {
      documentChanged() {},
      historyEntry() {},
      viewportChanged() {},
    });
    const contentHeight = createNodeContentHeightBatch(model, (id) =>
      id === "text"
        ? {
            id: "text",
            type: "text",
            data: createSpaceTextPayload("Text"),
            initialPosition: { x: 0, y: 0 },
          }
        : undefined,
    );

    contentHeight.setPaused(true);
    contentHeight.fit("text", measurement(96));
    await Promise.resolve();
    expect(model.document.nodes[0]?.frame.height).toBe(220);
    contentHeight.setPaused(false);
    await Promise.resolve();
    expect(model.document.nodes[0]?.frame.height).toBe(96);
  });
});

function measurement(height: number) {
  return { height, typographyScale: 1, width: 240 };
}

function textDocument(): CanvasDocumentV1 {
  return {
    version: 1,
    viewport: { x: 0, y: 0, zoom: 1 },
    nodes: [
      {
        nodeId: "text",
        frame: { x: 0, y: 0, width: 240, height: 220, rotation: 0 },
        sizeAuthority: "frame",
        zIndex: 0,
      },
    ],
  };
}
