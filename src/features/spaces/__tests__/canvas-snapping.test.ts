import { describe, expect, it } from "vitest";

import { frameSnapCoordinates, nearestCanvasSnap } from "../canvas/konva/interaction/canvas-snapping";
import { createCanvasTranslationSnapIndex } from "../canvas/konva/interaction/canvas-translation-snap-index";
import { snapCanvasTranslation } from "../canvas/konva/interaction/canvas-translation-snapping";
import type { CanvasNodeFrame } from "../canvas/schema/document";

describe("canvas snapping", () => {
  it("aligns node edges within a canvas-space tolerance", () => {
    const snappedTranslation = snapCanvasTranslation(
      [frame(0, 0, 100, 80)],
      createCanvasTranslationSnapIndex([frame(200, 30, 100, 80)]),
      { x: 96, y: 0 },
      5,
    );

    expect(snappedTranslation.x).toBe(100);
    expect(snappedTranslation.guides).toContainEqual({ kind: "alignment", points: [200, 0, 200, 110] });
  });

  it("centers a node between aligned neighbors with equal spacing guides", () => {
    const snappedTranslation = snapCanvasTranslation(
      [frame(101, 0, 50, 50)],
      createCanvasTranslationSnapIndex([frame(0, 0, 50, 50), frame(200, 0, 50, 50)]),
      { x: 0, y: 0 },
      2,
    );

    expect(snappedTranslation.x).toBe(-1);
    expect(snappedTranslation.guides.filter(({ kind }) => kind === "spacing")).toHaveLength(2);
  });

  it("chooses spacing neighbors by main-axis distance across staggered overlaps", () => {
    const snappedTranslation = snapCanvasTranslation(
      [frame(120, 0, 50, 100)],
      createCanvasTranslationSnapIndex([
        frame(40, 90, 50, 40),
        frame(20, 0, 50, 100),
        frame(210, 0, 50, 100),
      ]),
      { x: 0, y: 0 },
      6,
    );

    expect(snappedTranslation.x).toBe(5);
    expect(snappedTranslation.guides.filter(({ kind }) => kind === "spacing")).toHaveLength(2);
  });

  it("preserves equal-gap snapping between distant authored neighbors", () => {
    const result = snapCanvasTranslation(
      [frame(-49, 0, 100, 50)],
      createCanvasTranslationSnapIndex([frame(-5100, 0, 100, 50), frame(5000, 0, 100, 50)]),
      { x: 0, y: 0 },
      6,
    );
    expect(result.x).toBe(-1);
    expect(result.guides.filter(({ kind }) => kind === "spacing")).toHaveLength(2);
  });

  it("provides resize snap points for every node edge and center", () => {
    expect(frameSnapCoordinates([frame(20, 30, 100, 80)])).toEqual({
      x: [20, 70, 120],
      y: [30, 70, 110],
    });
    expect(nearestCanvasSnap(126, [120, 180], 6)).toBe(120);
    expect(nearestCanvasSnap(127, [120, 180], 6)).toBeUndefined();
  });
});

function frame(x: number, y: number, width: number, height: number): CanvasNodeFrame {
  return { x, y, width, height, rotation: 0 };
}
