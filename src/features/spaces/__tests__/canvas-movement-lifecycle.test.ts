import { describe, expect, it, vi } from "vitest";

import { movementHarness } from "../canvas/konva/testing/canvas-movement-harness";
import { movementScene } from "../testing/canvas-movement-fixture";

function draggedFrames(harness: ReturnType<typeof movementHarness>, x: number) {
  return harness.model.document.nodes.map(({ nodeId, frame }) => ({
    nodeId,
    frame: { ...frame, x: frame.x + x },
  }));
}

describe("movement gesture completion", () => {
  it("commits multiple dragged Sections without separately reparenting their members", async () => {
    const harness = movementHarness(movementScene());
    const updates = draggedFrames(harness, 100);
    vi.spyOn(harness.session.controller, "endDrag").mockResolvedValueOnce(updates);
    harness.session.beginDirectManipulation("drag");
    await harness.session.finish({ kind: "drag", nodeId: "s2" });
    expect(harness.events.onCommitGesture).toHaveBeenCalledWith(expect.objectContaining({ assignments: [] }));
    expect(harness.model.document.nodes.map(({ frame }) => frame.x)).toEqual([100, 1100, 140, 1180]);
    expect(harness.documentChanged).not.toHaveBeenCalled();
    expect(harness.session.directManipulation.value).toBeUndefined();
  });

  it("still permits an explicit content drag to leave its Section", async () => {
    const harness = movementHarness(movementScene(), new Set(["a"]));
    const node = harness.model.document.nodes.find(({ nodeId }) => nodeId === "a")!;
    vi.spyOn(harness.session.controller, "endDrag").mockResolvedValueOnce([
      { nodeId: "a", frame: { ...node.frame, x: 2500 } },
    ]);
    harness.session.beginDirectManipulation("drag");
    await harness.session.finish({ kind: "drag", nodeId: "a" });
    expect(harness.events.onCommitGesture).toHaveBeenCalledWith(
      expect.objectContaining({ assignments: [{ id: "a" }] }),
    );
  });

  it("does not change membership when a resize moves a content center outside its Section", async () => {
    const harness = movementHarness(movementScene(), new Set(["a"]));
    const node = harness.model.document.nodes.find(({ nodeId }) => nodeId === "a")!;
    vi.spyOn(harness.session.controller, "endTransform").mockResolvedValueOnce([
      { nodeId: "a", frame: { ...node.frame, width: 2000 } },
    ]);
    await harness.session.finish({ kind: "transform" });
    expect(harness.events.onCommitGesture).toHaveBeenCalledWith(expect.objectContaining({ assignments: [] }));
  });

  it("does not record an empty selection or a zero-offset movement", async () => {
    const harness = movementHarness(movementScene());
    await harness.session.nudge({ x: 0, y: 0 });
    harness.selection.clear();
    await harness.session.nudge({ x: 10, y: 10 });
    expect(harness.events.onCommitGesture).not.toHaveBeenCalled();
    expect(harness.events.onHistoryEntry).not.toHaveBeenCalled();
    expect(harness.session.busy.value).toBe(false);
  });

  it("rejects a missing descendant layout before any persisted keyboard movement", async () => {
    const harness = movementHarness(movementScene());
    const before = {
      ...harness.model.document,
      nodes: harness.model.document.nodes.filter(({ nodeId }) => nodeId !== "a"),
    };
    harness.model.replace(before);
    await expect(harness.session.nudge({ x: 5, y: 0 })).rejects.toThrow(/no layout/u);
    expect(harness.model.document).toEqual(before);
    expect(harness.events.onCommitGesture).not.toHaveBeenCalled();
  });
});
