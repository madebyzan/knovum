import { describe, expect, it } from "vitest";

import { createCanvasTranslationSnapIndex } from "../konva/interaction/canvas-translation-snap-index";
import { snapCanvasTranslation } from "../konva/interaction/canvas-translation-snapping";
import type { CanvasNodeFrame } from "../schema/document";

function frame(x: number, y = 0): CanvasNodeFrame {
  return { x, y, width: 100, height: 50, rotation: 0 };
}

function snap(moving: CanvasNodeFrame, stationary: CanvasNodeFrame[]) {
  return snapCanvasTranslation([moving], createCanvasTranslationSnapIndex(stationary), { x: 0, y: 0 }, 6);
}

function gaps(result: ReturnType<typeof snap>) {
  return result.guides
    .filter(({ kind }) => kind === "spacing")
    .map(({ points: [x1, y1, x2, y2] }) => Math.hypot(x2 - x1, y2 - y1));
}

describe("equal spacing", () => {
  it("continues an existing gap to the right", () => {
    const result = snap(frame(303), [frame(0), frame(150)]);
    expect(result.x).toBe(-3);
    expect(gaps(result)).toEqual([50, 50]);
  });

  it("continues an existing gap to the left", () => {
    const result = snap(frame(-148), [frame(0), frame(150)]);
    expect(result.x).toBe(-2);
    expect(gaps(result)).toEqual([50, 50]);
  });

  it("continues vertical spacing with unequal node sizes", () => {
    const result = snap(frame(0, 183), [frame(0, 0), frame(0, 90)]);
    expect(result.y).toBe(-3);
    expect(gaps(result)).toEqual([40, 40]);
  });

  it("does not borrow a gap from a different row", () => {
    const result = snap(frame(303, 70), [frame(0, 0), frame(150, 40)]);
    expect(result.x).toBe(0);
    expect(gaps(result)).toEqual([]);
  });

  it("does not create spacing guides for touching objects", () => {
    expect(gaps(snap(frame(203), [frame(0), frame(100)]))).toEqual([]);
  });

  it("chooses a closer alignment rather than forcing spacing", () => {
    const result = snap(frame(303), [frame(0), frame(150), frame(302, 200)]);
    expect(result.x).toBe(-1);
    expect(result.guides.some(({ kind }) => kind === "alignment")).toBe(true);
  });

  it("does not pull a node beyond the screen-space tolerance", () => {
    expect(snap(frame(307), [frame(0), frame(150)]).x).toBe(0);
  });
  it("draws staggered spacing through the actual overlap of all participants", () => {
    const result = snap(frame(303, 18), [frame(0, 0), frame(150, 10)]);
    const spacing = result.guides.filter(({ kind }) => kind === "spacing");
    expect(spacing).toHaveLength(2);
    expect(spacing.every(({ points }) => points[1] === 34 && points[3] === 34)).toBe(true);
  });

  it("reports the final vertical position after simultaneous two-axis snapping", () => {
    const result = snap(frame(303, 2), [frame(0), frame(150)]);
    expect(result.y).toBe(-2);
    expect(
      result.guides.filter(({ kind }) => kind === "spacing").every(({ points }) => points[1] === 25),
    ).toBe(true);
  });

  it("uses the moving group's outer bounds without changing internal spacing", () => {
    const result = snapCanvasTranslation(
      [frame(303), frame(413)],
      createCanvasTranslationSnapIndex([frame(0), frame(150)]),
      { x: 0, y: 0 },
      6,
    );
    expect(result.x).toBe(-3);
    expect(gaps(result)).toEqual([50, 50]);
  });

  it("keeps the snap radius in screen pixels when the gesture converts zoom", () => {
    for (const zoom of [0.25, 0.5, 1, 2]) {
      const index = createCanvasTranslationSnapIndex([frame(0), frame(150)]);
      const result = snapCanvasTranslation([frame(300 + 5 / zoom)], index, { x: 0, y: 0 }, 6 / zoom);
      expect(Math.abs(result.x * zoom)).toBeCloseTo(5);
    }
  });
});
