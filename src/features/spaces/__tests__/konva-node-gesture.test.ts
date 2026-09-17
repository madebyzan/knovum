import type Konva from "konva";
import { describe, expect, it, vi } from "vitest";

import type { CanvasNodeFrameUpdate, CanvasNodeLayoutV1 } from "../canvas/contract/canvas-document";
import { NodeGestureController } from "../canvas/konva/interaction/node-gesture-controller";
import { createSpaceNotePayload } from "../editor/tiptap-content";
import type { SpaceCanvasNode } from "../model/nodes/canvas-node";

describe("Konva node gestures", () => {
  it("moves a section and its domain members in one frame commit", async () => {
    const layouts = [layout("section", 0, 0), layout("member", 40, 80)];
    const nodes: SpaceCanvasNode[] = [
      {
        id: "section",
        type: "section",
        data: { kind: "section", title: "Section" },
        initialPosition: { x: 0, y: 0 },
      },
      {
        id: "member",
        type: "note",
        data: { ...createSpaceNotePayload("Member", ""), sectionId: "section" },
        initialPosition: { x: 40, y: 80 },
      },
    ];
    const shapes = new Map([
      ["section", shape("section", 0, 0)],
      ["member", shape("member", 40, 80)],
    ]);
    const stage = fakeStage(shapes);
    const previewFrames = vi.fn<(updates: readonly CanvasNodeFrameUpdate[]) => void>();
    const gestures = new NodeGestureController({
      layouts: () => layouts,
      nodes: () => nodes,
      previewFrames,
      selectedNodeIds: () => new Set(),
      stage: () => stage,
      updateFeedback() {},
      zoom: () => 1,
    });

    gestures.beginDrag("section");
    shapes.get("section")!.position({ x: 100, y: 60 });
    gestures.moveDrag("section");
    const updates = await gestures.endDrag("section");

    expect(shapes.get("member")!.position()).toEqual({ x: 140, y: 140 });
    expect(previewFrames).toHaveBeenCalledWith(
      expect.arrayContaining([expect.objectContaining({ nodeId: "member" })]),
    );
    expect(updates).toHaveLength(2);
  });

  it("does not move locked nodes in a multi-selection", async () => {
    const layouts = [layout("free", 0, 0), layout("locked", 40, 80)];
    const nodes = [
      canvasNote("free", 0, 0),
      { ...canvasNote("locked", 40, 80), data: { ...createSpaceNotePayload("Locked", ""), locked: true } },
    ];
    const shapes = new Map([
      ["free", shape("free", 0, 0)],
      ["locked", shape("locked", 40, 80)],
    ]);
    const stage = fakeStage(shapes);
    const gestures = new NodeGestureController({
      layouts: () => layouts,
      nodes: () => nodes,
      previewFrames() {},
      selectedNodeIds: () => new Set(["free", "locked"]),
      stage: () => stage,
      updateFeedback() {},
      zoom: () => 1,
    });

    gestures.beginDrag("free");
    shapes.get("free")!.position({ x: 20, y: 20 });
    gestures.moveDrag("free");
    const updates = await gestures.endDrag("free");

    expect(shapes.get("locked")!.position()).toEqual({ x: 40, y: 80 });
    expect(updates.map(({ nodeId }) => nodeId)).toEqual(["free"]);
  });

  it("normalizes the native Transformer scale when the gesture ends", async () => {
    const transformed = transformShape("note", { x: 20, y: 30, scaleX: 1.5, scaleY: 2 });
    const gestures = new NodeGestureController({
      layouts: () => [layout("note", 0, 0)],
      nodes: () => [canvasNote("note", 0, 0)],
      previewFrames() {},
      selectedNodeIds: () => new Set(["note"]),
      stage: () => fakeStage(new Map([["note", transformed]])),
      updateFeedback() {},
      zoom: () => 1,
    });

    gestures.beginTransform();
    const updates = await gestures.endTransform();

    expect(updates[0]?.frame).toEqual({ x: 20, y: 30, width: 450, height: 440, rotation: 0 });
    expect(updates[0]?.sizeAuthority).toBe("frame");
    expect(transformed.scaleX()).toBe(1);
    expect(transformed.scaleY()).toBe(1);
  });
});

function canvasNote(id: string, x: number, y: number): SpaceCanvasNode {
  return { id, type: "note", data: createSpaceNotePayload(id, ""), initialPosition: { x, y } };
}

function fakeStage(shapes: ReadonlyMap<string, FakeNode>) {
  return {
    find: (predicate: (node: FakeNode) => boolean) => [...shapes.values()].filter(predicate),
    findOne: (predicate: (node: FakeNode) => boolean) => [...shapes.values()].find(predicate),
  } as unknown as Konva.Stage;
}

function layout(nodeId: string, x: number, y: number): CanvasNodeLayoutV1 {
  return {
    nodeId,
    frame: { x, y, width: 300, height: 220, rotation: 0 },
    sizeAuthority: "frame",
    zIndex: 0,
  };
}

interface FakeNode {
  getAttr(key: string): string;
  x(): number;
  y(): number;
  position(): { x: number; y: number };
  position(point: { x: number; y: number }): void;
  rotation(): number;
  rotation(value: number): void;
  scale(point: { x: number; y: number }): void;
  scaleX(): number;
  scaleY(): number;
}

function shape(nodeId: string, initialX: number, initialY: number): FakeNode {
  let point = { x: initialX, y: initialY };
  return {
    getAttr: () => nodeId,
    x: () => point.x,
    y: () => point.y,
    position(next?: { x: number; y: number }) {
      if (next) point = next;
      return point;
    },
    rotation: () => 0,
    scale() {},
    scaleX: () => 1,
    scaleY: () => 1,
  } as FakeNode;
}

function transformShape(
  nodeId: string,
  transform: { x: number; y: number; scaleX: number; scaleY: number },
): FakeNode {
  const node = shape(nodeId, transform.x, transform.y);
  let scale = { x: transform.scaleX, y: transform.scaleY };
  return {
    ...node,
    scale(next) {
      scale = next;
    },
    scaleX: () => scale.x,
    scaleY: () => scale.y,
  } as FakeNode;
}
