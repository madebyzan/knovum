import { describe, expect, it } from "vitest";

import { viewportToLocalPoint } from "../viewport-to-local-point";

function target(
  width: number,
  height: number,
  bounds: { left: number; top: number; width: number; height: number },
) {
  return {
    offsetWidth: width,
    offsetHeight: height,
    getBoundingClientRect: () => bounds,
  } as HTMLElement;
}

describe("canvas DOM pointer coordinates", () => {
  it("inverts viewport zoom on an unrotated node", () => {
    const element = target(100, 50, { left: 300, top: 400, width: 200, height: 100 });
    expect(viewportToLocalPoint(element, 350, 420, 0)).toEqual({ x: 25, y: 10 });
  });

  it("inverts node rotation and viewport zoom together", () => {
    const element = target(100, 50, { left: 200, top: 400, width: 100, height: 200 });
    const point = viewportToLocalPoint(element, 280, 450, 90);
    expect(point.x).toBeCloseTo(25);
    expect(point.y).toBeCloseTo(10);
  });
});
