import { describe, expect, it } from "vitest";

import { commandForCanvasGesture } from "../canvas/document/structure-gesture-command";
import type { CanvasDocumentV1 } from "../canvas/schema/document";
import type { SpaceEditSnapshot } from "../model/structure/schema/edit-snapshot";

describe("commandForCanvasGesture appearance comparison", () => {
  it("treats appearance key order alone as no edit", () => {
    const before = adapterDocument({ fill: "yellow", fontSize: 16 });
    const after = adapterDocument({ fontSize: 16, fill: "yellow" });

    expect(commandForCanvasGesture(snapshot(), before, after)).toBeUndefined();
  });

  it("emits one layout command when an appearance value actually changes", () => {
    const before = adapterDocument({ fill: "yellow", fontSize: 16 });
    const after = adapterDocument({ fill: "blue", fontSize: 16 });

    expect(commandForCanvasGesture(snapshot(), before, after)).toEqual({
      kind: "content.update-layout",
      structureNodeId: "content-1",
      transform: { x: 100, y: 50, rotation: 0, scale: 1 },
      size: { width: 200, height: 120 },
      sizeAuthority: "frame",
      appearance: { kind: "set", value: { fill: "blue", fontSize: 16 } },
    });
  });
});

function adapterDocument(appearance: CanvasDocumentV1["nodes"][number]["appearance"]): CanvasDocumentV1 {
  return {
    version: 1,
    viewport: { x: 0, y: 0, zoom: 1 },
    nodes: [
      {
        nodeId: "node-1",
        frame: { x: 100, y: 50, width: 200, height: 120, rotation: 0 },
        sizeAuthority: "frame",
        appearance,
        zIndex: 0,
      },
    ],
  };
}

function snapshot(): SpaceEditSnapshot {
  return {
    editRevision: 1,
    relationStyles: [],
    structure: [
      { id: "content-1", kind: "content", parentId: null, order: 0, locked: false, nodeId: "node-1" },
    ],
    layouts: [
      {
        structureNodeId: "content-1",
        kind: "content",
        transform: { x: 100, y: 50, rotation: 0, scale: 1 },
        size: { width: 200, height: 120 },
        sizeAuthority: "frame",
      },
    ],
  };
}
