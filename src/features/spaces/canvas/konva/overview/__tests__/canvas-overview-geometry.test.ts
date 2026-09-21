import { describe, expect, it } from "vitest";

import { createSpaceNotePayload } from "../../../../editor/tiptap-content";
import type { CanvasDocumentV1 } from "../../../contract/canvas-document";
import { projectCanvasScene } from "../../scene/scene-projection";
import { canvasOverviewViewport } from "../canvas-overview-geometry";

function sceneAt(frame: { x: number; y: number; width: number; height: number; rotation?: number }) {
  const { x, y, width, height, rotation = 0 } = frame;
  const document: CanvasDocumentV1 = {
    version: 1,
    viewport: { x: 0, y: 0, zoom: 1 },
    nodes: [
      {
        nodeId: "note",
        frame: { x, y, width, height, rotation },
        sizeAuthority: "frame",
        zIndex: 0,
      },
    ],
  };
  return projectCanvasScene(
    document,
    [
      {
        id: "note",
        type: "note",
        data: createSpaceNotePayload("Note", ""),
        initialPosition: { x, y },
      },
    ],
    [],
  );
}

describe("canvas overview geometry", () => {
  it("fits the whole document independently of editor viewport", () => {
    const scene = sceneAt({ x: -500, y: 700, width: 1000, height: 500 });
    expect(canvasOverviewViewport(scene, 480, 270)).toEqual({ x: 240, y: -283, scale: 0.44 });
  });

  it("includes rotated node corners", () => {
    const viewport = canvasOverviewViewport(
      sceneAt({ x: 0, y: 0, width: 100, height: 200, rotation: 90 }),
      480,
      270,
    );
    expect(viewport.scale).toBe(1);
    expect(viewport.x).toBe(340);
    expect(viewport.y).toBe(85);
  });

  it("does not depend on a mounted viewport when empty", () => {
    const scene = projectCanvasScene(
      { version: 1, viewport: { x: 900, y: 900, zoom: 0.25 }, nodes: [] },
      [],
      [],
    );
    expect(canvasOverviewViewport(scene, 480, 270)).toEqual({ x: 0, y: 0, scale: 1 });
  });
});
