import { describe, expect, it, vi } from "vitest";

import type { CanvasHistoryEntry } from "../canvas/contract/canvas-editor";
import { movementHarness } from "../canvas/konva/testing/canvas-movement-harness";
import { movementScene } from "../testing/canvas-movement-fixture";

function xPositions(harness: ReturnType<typeof movementHarness>) {
  return harness.model.document.nodes.map(({ nodeId, frame }) => [nodeId, frame.x]);
}

describe("wired movement and persistence semantics", () => {
  it("aligns two Sections and both member sets through the real editor port", async () => {
    const harness = movementHarness(movementScene());
    await harness.port.alignSelection("left");
    expect(xPositions(harness)).toEqual([
      ["s1", 0],
      ["s2", 0],
      ["a", 40],
      ["b", 80],
    ]);
    expect(harness.events.onCommitGesture).toHaveBeenCalledOnce();
    expect(harness.events.onCommitGesture).toHaveBeenCalledWith(expect.objectContaining({ assignments: [] }));
  });
  it("does not realign a selected member separately from its Section", async () => {
    const harness = movementHarness(movementScene(), new Set(["s1", "s2", "b"]));
    await harness.port.alignSelection("left");
    expect(xPositions(harness)).toEqual([
      ["s1", 0],
      ["s2", 0],
      ["a", 40],
      ["b", 80],
    ]);
  });
  it("moves multiple Sections with keyboard input and does not rewrite memberships", async () => {
    const harness = movementHarness(movementScene());
    await harness.session.nudge({ x: 20, y: 15 });
    expect(xPositions(harness)).toEqual([
      ["s1", 20],
      ["s2", 1_020],
      ["a", 60],
      ["b", 1_100],
    ]);
    expect(harness.events.onCommitGesture).toHaveBeenCalledWith(expect.objectContaining({ assignments: [] }));
  });
  it("keeps locked descendants attached to an unlocked Section for keyboard input", async () => {
    const nodes = movementScene();
    nodes[2]!.data = { ...nodes[2]!.data, locked: true };
    const harness = movementHarness(nodes, new Set(["s1", "a"]));
    await harness.session.nudge({ x: 30, y: 0 });
    expect(xPositions(harness)).toEqual([
      ["s1", 30],
      ["s2", 1_000],
      ["a", 70],
      ["b", 1_080],
    ]);
  });
  it("rejects an entire nudge against a locked operation root", async () => {
    const nodes = movementScene();
    nodes[1]!.data = { ...nodes[1]!.data, locked: true };
    const harness = movementHarness(nodes);
    const before = structuredClone(harness.model.document);
    await expect(harness.session.nudge({ x: 30, y: 0 })).rejects.toThrow(/Unlock/u);
    expect(harness.model.document).toEqual(before);
    expect(harness.events.onCommitGesture).not.toHaveBeenCalled();
    expect(harness.session.busy.value).toBe(false);
  });
  it("keeps fit separate from moving a Section and does not adopt overlapping content", async () => {
    const harness = movementHarness(movementScene(), new Set(["s1"]));
    const member = structuredClone(harness.model.document.nodes.find(({ nodeId }) => nodeId === "a"));
    await harness.port.fitSection("s1");
    expect(harness.model.document.nodes.find(({ nodeId }) => nodeId === "a")).toEqual(member);
    expect(harness.events.onCommitGesture).toHaveBeenCalledWith(expect.objectContaining({ assignments: [] }));
  });
  it("never sends an optimistic gesture or a rejected gesture to the independent autosave path", async () => {
    const harness = movementHarness(movementScene());
    const before = structuredClone(harness.model.document);
    vi.mocked(harness.events.onCommitGesture).mockRejectedValueOnce(new Error("revision conflict"));
    await expect(harness.session.nudge({ x: 30, y: 0 })).rejects.toThrow(/revision conflict/u);
    expect(harness.model.document).toEqual(before);
    expect(harness.documentChanged).not.toHaveBeenCalled();
    expect(harness.events.onHistoryEntry).not.toHaveBeenCalled();
    expect(harness.events.onMutationPendingChange).toHaveBeenLastCalledWith(false);
  });
  it("records and undoes the entire aligned operation while preserving a newer camera", async () => {
    const harness = movementHarness(movementScene());
    const before = xPositions(harness);
    await harness.port.alignSelection("left");
    const entry: CanvasHistoryEntry = vi.mocked(harness.events.onHistoryEntry).mock.calls[0]![0];
    harness.model.setViewport({ x: 17, y: 31, zoom: 2 });
    harness.documentChanged.mockClear();
    await entry.undo();
    expect(xPositions(harness)).toEqual(before);
    expect(harness.model.document.viewport).toEqual({ x: 17, y: 31, zoom: 2 });
    expect(harness.documentChanged).not.toHaveBeenCalled();
    await entry.redo();
    expect(xPositions(harness)).toEqual([
      ["s1", 0],
      ["s2", 0],
      ["a", 40],
      ["b", 80],
    ]);
  });
});
