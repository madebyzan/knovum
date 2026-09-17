import { describe, expect, it } from "vitest";

import { constrainSelectionTransform } from "../canvas/konva/selection/selection-transform-geometry";
import type { CanvasSelectionTransformPolicy } from "../canvas/konva/selection/selection-transform-policy";

const policy: CanvasSelectionTransformPolicy = {
  nodeIds: ["text"],
  resizeMode: "width-and-proportional",
  minWidth: 96,
  minHeight: 40,
  maxWidth: 960,
  maxHeight: 560,
  proportionalScale: { baseWidth: 240, baseHeight: 64, min: 0.25, max: 4 },
};
const current = { x: 100, y: 100, width: 240, height: 64, rotation: 0 };
const scale = { x: 1, y: 1 };

describe("Konva transform geometry", () => {
  it("changes only width from a text side handle", () => {
    expect(
      constrainSelectionTransform(current, { ...current, width: 320 }, "middle-right", policy, scale),
    ).toEqual({ ...current, width: 320 });
  });

  it("preserves the current text ratio from its bottom-right scale handle", () => {
    expect(
      constrainSelectionTransform(
        current,
        { ...current, width: 480, height: 128 },
        "bottom-right",
        policy,
        scale,
      ),
    ).toEqual({ ...current, width: 480, height: 128 });
  });

  it("scales text below its reflow minimum from the proportional handle", () => {
    expect(
      constrainSelectionTransform(
        current,
        { ...current, width: 60, height: 16 },
        "bottom-right",
        policy,
        scale,
      ),
    ).toEqual({ ...current, width: 60, height: 16 });
  });

  it("projects noisy pointer movement onto one stable proportional axis", () => {
    const first = constrainSelectionTransform(
      current,
      { ...current, width: 360, height: 129 },
      "bottom-right",
      policy,
      scale,
    );
    const second = constrainSelectionTransform(
      current,
      { ...current, width: 361, height: 95 },
      "bottom-right",
      policy,
      scale,
    );
    expect(first.width / first.height).toBeCloseTo(240 / 64);
    expect(second.width / second.height).toBeCloseTo(240 / 64);
    expect(first.width).toBeGreaterThan(360);
    expect(second.width).toBeLessThan(361);
  });

  it("responds naturally when a proportional handle moves mostly vertically", () => {
    const result = constrainSelectionTransform(
      current,
      { ...current, height: 128 },
      "bottom-right",
      policy,
      scale,
    );
    expect(result.width).toBeGreaterThan(current.width);
    expect(result.width / result.height).toBeCloseTo(240 / 64);
  });

  it("projects fixed-caption media cards onto their content aspect", () => {
    const videoPolicy: CanvasSelectionTransformPolicy = {
      nodeIds: ["video"],
      resizeMode: "proportional",
      minWidth: 200,
      minHeight: 164,
      maxWidth: 920,
      maxHeight: 972,
      contentAspect: { ratio: 16 / 9, fixedVerticalChrome: 64 },
    };
    const video = { x: 0, y: 0, width: 480, height: 334, rotation: 0 };
    const result = constrainSelectionTransform(
      video,
      { ...video, height: 604 },
      "bottom-right",
      videoPolicy,
      scale,
    );

    expect(result.width).toBeGreaterThan(video.width);
    expect(result.height).toBeCloseTo(result.width / (16 / 9) + 64);
  });
});
