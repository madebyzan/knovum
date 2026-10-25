import { describe, expect, it } from "vitest";

import { ComparisonFrame } from "../comparison/comparison-view";

const STAGE = { width: 1096, height: 596 };
const WIDE = new ComparisonFrame(STAGE, { width: 4000, height: 1000 });

describe("comparison frame", () => {
  it("fits the whole picture inside the stage's margin, centred", () => {
    expect(WIDE.fitted).toEqual({ scale: 0.25, x: 48, y: 173 });
  });

  it("never enlarges a small picture past its own resolution", () => {
    const small = new ComparisonFrame(STAGE, { width: 100, height: 50 });
    expect(small.fitted).toEqual(small.actualSize);
    expect(small.actualSize).toEqual({ scale: 1, x: 498, y: 273 });
  });

  it("shows actual size from the picture's centre", () => {
    expect(WIDE.actualSize).toEqual({ scale: 1, x: -1452, y: -202 });
  });

  it("keeps the picture pixel under the pointer in place while zooming", () => {
    const point = { x: 300, y: 300 };
    const from = WIDE.actualSize;
    const zoomed = WIDE.zoomed(from, point, 2);
    const under = (view: typeof from) => [(point.x - view.x) / view.scale, (point.y - view.y) / view.scale];
    expect(zoomed.scale).toBe(2);
    expect(under(zoomed)).toEqual(under(from));
  });

  it("zooms out no further than the fitted view, and in no further than eight stage pixels a pixel", () => {
    expect(WIDE.zoomed(WIDE.fitted, { x: 0, y: 0 }, 0.1)).toEqual(WIDE.fitted);
    expect(WIDE.zoomed(WIDE.fitted, { x: 0, y: 0 }, 1000).scale).toBe(8);
  });

  it("pans a larger picture to its edges and no further, and keeps a smaller one centred", () => {
    expect(WIDE.panned(WIDE.actualSize, 10_000, -10_000)).toEqual({ scale: 1, x: 0, y: -404 });
    expect(WIDE.panned(WIDE.fitted, 300, 300)).toEqual(WIDE.fitted);
  });

  it("measures the part of the stage the picture covers", () => {
    expect(WIDE.span(WIDE.fitted)).toEqual({ left: 48, right: 1048, top: 173, bottom: 423 });
    expect(WIDE.span(WIDE.actualSize)).toEqual({ left: 0, right: 1096, top: 0, bottom: 596 });
  });

  it("steps zoom through powers of two, stopping at the fitted view", () => {
    const steps = [WIDE.stepped(WIDE.fitted, 1)];
    steps.push(
      WIDE.stepped(steps[0]!, 1),
      WIDE.stepped(WIDE.actualSize, -1),
      WIDE.stepped(WIDE.stepped(WIDE.actualSize, -1), -1),
    );
    expect(steps.map(({ scale }) => scale)).toEqual([0.5, 1, 0.5, 0.25]);
    expect(WIDE.canZoomOut(WIDE.fitted)).toBe(false);
    expect(WIDE.canZoomIn(WIDE.zoomed(WIDE.fitted, { x: 0, y: 0 }, 1000))).toBe(false);
  });
});
