import { describe, expect, it } from "vitest";

import { createCanvasDocument } from "../canvas/document/canvas-document-sync";
import { expandCanvasMovement } from "../canvas/document/canvas-movement";
import { captureCanvasNodeDragState } from "../canvas/konva/interaction/canvas-node-drag-state";
import { movementScene } from "../testing/canvas-movement-fixture";

describe("container movement plans", () => {
  it.each(["s1", "s2"])("captures all selected Section members when %s starts the drag", (primaryId) => {
    const nodes = movementScene();
    const layouts = createCanvasDocument(nodes).nodes;
    const result = captureCanvasNodeDragState({
      layouts,
      nodes,
      primaryId,
      selectedNodeIds: new Set(["s1", "s2"]),
      stage: undefined,
    });
    expect(new Set(result!.state.frames.keys())).toEqual(new Set(["s1", "s2", "a", "b"]));
    expect(result!.state.origin).toEqual(layouts.find(({ nodeId }) => nodeId === primaryId)?.frame);
  });
  it("copies every captured frame and retains locked descendants", () => {
    const nodes = movementScene();
    nodes[3]!.data = { ...nodes[3]!.data, locked: true };
    const layouts = createCanvasDocument(nodes).nodes;
    const result = captureCanvasNodeDragState({
      layouts,
      nodes,
      primaryId: "s1",
      selectedNodeIds: new Set(["s1", "s2"]),
      stage: undefined,
    });
    expect(result!.state.frames.has("b")).toBe(true);
    expect(result!.state.frames.get("b")).not.toBe(layouts[3]!.frame);
  });
  it("rejects capture rather than skipping a locked selected root", () => {
    const nodes = movementScene();
    nodes[1]!.data = { ...nodes[1]!.data, locked: true };
    expect(
      captureCanvasNodeDragState({
        nodes,
        layouts: createCanvasDocument(nodes).nodes,
        primaryId: "s1",
        selectedNodeIds: new Set(["s1", "s2"]),
        stage: undefined,
      }),
    ).toBeUndefined();
  });
  it("applies a different offset to each whole root for alignment/distribution", () => {
    const nodes = movementScene();
    const layouts = createCanvasDocument(nodes).nodes;
    const updates = layouts.slice(0, 2).map(({ nodeId, frame }) => ({ nodeId, frame: { ...frame, x: 500 } }));
    const result = expandCanvasMovement(nodes, layouts, updates);
    const frames = new Map(result.map(({ nodeId, frame }) => [nodeId, frame]));
    expect(frames.get("s1")!.x).toBe(500);
    expect(frames.get("a")!.x).toBe(540);
    expect(frames.get("s2")!.x).toBe(500);
    expect(frames.get("b")!.x).toBe(580);
  });
  it("moves a selected child only once when its Section is also selected", () => {
    const nodes = movementScene();
    const layouts = createCanvasDocument(nodes).nodes;
    const selected = layouts.filter(({ nodeId }) => ["s1", "a"].includes(nodeId));
    const updates = selected.map(({ nodeId, frame }) => ({ nodeId, frame: { ...frame, x: frame.x + 25 } }));
    const result = expandCanvasMovement(nodes, layouts, updates);
    expect(result).toHaveLength(2);
    expect(result.find(({ nodeId }) => nodeId === "a")!.frame.x).toBe(65);
  });
  it("does not interpret resizing as a movement", () => {
    const nodes = movementScene();
    const layouts = createCanvasDocument(nodes).nodes;
    const section = layouts[0]!;
    expect(() =>
      expandCanvasMovement(nodes, layouts, [
        { nodeId: section.nodeId, frame: { ...section.frame, width: 900 } },
      ]),
    ).toThrow(/cannot resize/u);
  });
  it("rejects missing descendant geometry instead of silently leaving it behind", () => {
    const nodes = movementScene();
    const layouts = createCanvasDocument(nodes).nodes.filter(({ nodeId }) => nodeId !== "a");
    expect(() =>
      expandCanvasMovement(nodes, layouts, [{ nodeId: "s1", frame: { ...layouts[0]!.frame, x: 20 } }]),
    ).toThrow(/no layout/u);
  });
});
