import { describe, expect, it, vi } from "vitest";

import { autoLayoutCanvas } from "../canvas/document/canvas-auto-layout";
import { movementHarness } from "../canvas/konva/testing/canvas-movement-harness";
import { canvasNote } from "../testing/canvas-fixtures";
import { movementScene } from "../testing/canvas-movement-fixture";

vi.mock("../canvas/document/canvas-auto-layout", () => ({
  autoLayoutCanvas: vi.fn<typeof autoLayoutCanvas>(),
}));

describe("whole-operation arrangement through the canvas port", () => {
  it("distributes three Section roots and carries every member by its parent's delta", async () => {
    const nodes = movementScene();
    const child = canvasNote("c");
    child.initialPosition = { x: 2580, y: 100 };
    child.data = { ...child.data, sectionId: "s3", locked: true };
    nodes.push(
      {
        id: "s3",
        type: "section",
        data: { kind: "section", title: "C" },
        initialPosition: { x: 2500, y: 0 },
      },
      child,
    );
    const harness = movementHarness(nodes, new Set(["s3", "s2", "s1", "a"]));
    await harness.port.distributeSelection("horizontal");
    const positions = new Map(harness.model.document.nodes.map(({ nodeId, frame }) => [nodeId, frame.x]));
    expect(positions.get("s1")).toBe(0);
    expect(positions.get("s2")).toBe(1250);
    expect(positions.get("s3")).toBe(2500);
    expect(positions.get("a")).toBe(40);
    expect(positions.get("b")).toBe(1330);
    expect(positions.get("c")).toBe(2580);
  });

  it("gives ELK only operation roots and expands its resulting movement to members", async () => {
    const harness = movementHarness(movementScene(), new Set(["s1", "s2", "a"]));
    vi.mocked(autoLayoutCanvas).mockImplementationOnce(async (layouts) =>
      layouts.map(({ nodeId, frame }, index) => ({
        nodeId,
        frame: { ...frame, x: 200 + index * 600, y: 300 },
      })),
    );
    await harness.port.autoLayoutSelection("RIGHT");
    expect(vi.mocked(autoLayoutCanvas).mock.calls[0]![0].map(({ nodeId }) => nodeId)).toEqual(["s1", "s2"]);
    expect(harness.model.document.nodes.map(({ frame }) => [frame.x, frame.y])).toEqual([
      [200, 300],
      [800, 300],
      [240, 350],
      [880, 350],
    ]);
    expect(harness.events.onCommitGesture).toHaveBeenCalledOnce();
  });

  it("does not persist a partial alignment when any selected root is locked", async () => {
    const nodes = movementScene();
    nodes[1]!.data = { ...nodes[1]!.data, locked: true };
    const harness = movementHarness(nodes);
    const before = structuredClone(harness.model.document);
    expect(() => harness.port.alignSelection("left")).toThrow(/Unlock/u);
    expect(harness.model.document).toEqual(before);
    expect(harness.events.onCommitGesture).not.toHaveBeenCalled();
  });

  it("leaves the document unchanged if automatic layout computation fails", async () => {
    const harness = movementHarness(movementScene());
    const before = structuredClone(harness.model.document);
    vi.mocked(autoLayoutCanvas).mockRejectedValueOnce(new Error("Layout worker unavailable"));
    await expect(harness.port.autoLayoutSelection("DOWN")).rejects.toThrow(/worker unavailable/u);
    expect(harness.model.document).toEqual(before);
    expect(harness.events.onCommitGesture).not.toHaveBeenCalled();
    expect(harness.session.busy.value).toBe(false);
  });
});
