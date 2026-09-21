import type Konva from "konva";
import { describe, expect, it, vi } from "vitest";

import type { CanvasGestureUpdate } from "../canvas/konva/interaction/canvas-gesture-update";
import { NodeGestureController } from "../canvas/konva/interaction/node-gesture-controller";
import { createSpaceNotePayload } from "../editor/tiptap-content";

describe("Konva node gesture callbacks", () => {
  it("keeps event callbacks bound to their gesture controller", () => {
    const previewFrames = vi.fn<(updates: readonly CanvasGestureUpdate[]) => void>();
    const node = fakeNode();
    const gestures = new NodeGestureController({
      layouts: () => [
        {
          nodeId: "note",
          frame: { x: 0, y: 0, width: 300, height: 220, rotation: 0 },
          sizeAuthority: "frame",
          zIndex: 0,
        },
      ],
      nodes: () => [
        {
          id: "note",
          type: "note",
          data: createSpaceNotePayload("Note"),
          initialPosition: { x: 0, y: 0 },
        },
      ],
      lockedNodeIds: () => new Set(),
      previewFrames,
      selectedNodeIds: () => new Set(["note"]),
      stage: () => ({ find: () => [node], findOne: () => node }) as unknown as Konva.Stage,
      updateFeedback() {},
      zoom: () => 1,
    });
    gestures.beginTransform();
    const { beginDrag, moveDrag, previewTransform } = gestures;
    beginDrag("note");
    moveDrag("note");
    previewTransform();
    expect(previewFrames).toHaveBeenCalled();
  });
});

function fakeNode() {
  let point = { x: 0, y: 0 };
  return {
    getAttr: () => "note",
    x: () => point.x,
    y: () => point.y,
    position: (next?: { x: number; y: number }) => (next ? (point = next) : point),
    rotation: () => 0,
    scale: () => undefined,
    scaleX: () => 1,
    scaleY: () => 1,
  } as unknown as Konva.Node;
}
