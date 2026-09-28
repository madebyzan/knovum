import { describe, expect, it } from "vitest";

import { canvasBaseFontSize, effectiveCanvasFontSize } from "../canvas/document/canvas-typography";
import { scaledCanvasTextAppearance } from "../canvas/konva/interaction/canvas-text-transform";
import type { CanvasNodeLayoutV1 } from "../canvas/schema/document";

const layout: CanvasNodeLayoutV1 = {
  nodeId: "text",
  frame: { x: 0, y: 0, width: 240, height: 40, rotation: 0 },
  sizingMode: "frame",
  zIndex: 0,
  appearance: {
    fontSize: 18,
    fontStyle: "italic",
    typographyScale: 1.25,
  },
};

describe("canvas text transform", () => {
  it("scales the complete typography plane without rewriting text semantics", () => {
    expect(scaledCanvasTextAppearance(layout, 480)).toEqual({
      fontSize: 18,
      fontStyle: "italic",
      typographyScale: 2.5,
    });
  });

  it("derives every preview from the canonical gesture baseline", () => {
    expect(scaledCanvasTextAppearance(layout, 120).typographyScale).toBe(0.625);
    expect(scaledCanvasTextAppearance(layout, 360).typographyScale).toBe(1.875);
  });

  it("keeps floating-point gesture results inside the persisted typography domain", () => {
    expect(scaledCanvasTextAppearance(layout, 960).typographyScale).toBe(4);
    expect(scaledCanvasTextAppearance(layout, 24).typographyScale).toBe(0.25);
  });

  it("round-trips effective toolbar sizes through a scaled typography plane", () => {
    expect(effectiveCanvasFontSize(18, 2)).toBe(36);
    expect(canvasBaseFontSize(24, 2)).toBe(12);
  });
});
