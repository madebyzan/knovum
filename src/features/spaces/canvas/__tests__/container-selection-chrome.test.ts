import { assert, describe, expect, it } from "vitest";

import { canvasChromeMetrics } from "../canvas-chrome-metrics";
import { canvasSelectionGap } from "../presentation/selection/canvas-selection-gap";
import { placeSelectionSurface } from "../presentation/selection/selection-surface-placement";

describe("container selection chrome", () => {
  it.each([0.25, 1, 2])("keeps the action bar clear of a Group rotation handle at zoom %s", (zoom) => {
    const placement = placeSelectionSurface({
      selection: { left: 300, top: 300, right: 500, bottom: 500 },
      viewport: { width: 1000, height: 800 },
      insets: { top: 8, right: 8, bottom: 8, left: 8 },
      reserved: [],
      surface: { width: 240, height: 40 },
      kind: "actions",
      gap: canvasSelectionGap(undefined, true),
    });
    assert.isDefined(placement);
    expect(placement.side).toBe("above");
    const group = canvasChromeMetrics(zoom).group;
    const hitAreaTop = 300 - (group.rotationOffset + group.handleRadius + group.hitStrokeWidth / 2) * zoom;
    expect(placement.top + 40).toBeLessThanOrEqual(hitAreaTop - 4);
  });
});
