import { describe, expect, it } from "vitest";

import { actualSizeView, fittedView, zoomedView } from "../comparison/comparison-view";

const STAGE = { width: 1000, height: 500 };
const PICTURE = { width: 4000, height: 1000 };

describe("comparison view", () => {
  it("fits the whole picture centred in the stage", () => {
    expect(fittedView(STAGE, PICTURE)).toEqual({ scale: 0.25, x: 0, y: 125 });
  });

  it("keeps the picture pixel under the pointer in place while zooming", () => {
    const fitted = fittedView(STAGE, PICTURE);
    const point = { x: 300, y: 200 };
    const zoomed = zoomedView(fitted, point, 4, fitted);
    const under = (view: typeof fitted) => [(point.x - view.x) / view.scale, (point.y - view.y) / view.scale];
    expect(zoomed.scale).toBe(1);
    expect(under(zoomed)).toEqual(under(fitted));
  });

  it("zooms out no further than the fitted view, and in no further than eight stage pixels a pixel", () => {
    const fitted = fittedView(STAGE, PICTURE);
    expect(zoomedView(fitted, { x: 0, y: 0 }, 0.1, fitted).scale).toBe(0.25);
    expect(zoomedView(fitted, { x: 0, y: 0 }, 1000, fitted).scale).toBe(8);
  });

  it("lets a picture smaller than the stage be seen at its own resolution", () => {
    const small = { width: 100, height: 50 };
    const fitted = fittedView(STAGE, small);
    expect(fitted.scale).toBe(10);
    expect(zoomedView(fitted, { x: 0, y: 0 }, 0.01, fitted).scale).toBe(1);
    expect(actualSizeView(STAGE, small)).toEqual({ scale: 1, x: 450, y: 225 });
  });
});
