import { describe, expect, it } from "vitest";

import { edgeAutoPanVelocity } from "../edge-auto-pan";

describe("canvas edge auto-pan", () => {
  const size = { width: 800, height: 600 };

  it("stays still outside the edge activation zone", () => {
    expect(edgeAutoPanVelocity({ x: 400, y: 300 }, size)).toEqual({ x: 0, y: 0 });
  });

  it("accelerates toward the approached edge", () => {
    const near = edgeAutoPanVelocity({ x: 20, y: 300 }, size);
    const atEdge = edgeAutoPanVelocity({ x: 0, y: 300 }, size);
    expect(near.x).toBeLessThan(0);
    expect(Math.abs(atEdge.x)).toBeGreaterThan(Math.abs(near.x));
    expect(atEdge).toEqual({ x: -720, y: 0 });
  });

  it("supports diagonal panning at corners", () => {
    const velocity = edgeAutoPanVelocity({ x: 800, y: 600 }, size);
    expect(velocity).toEqual({ x: 720, y: 720 });
  });
});
