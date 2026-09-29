import { describe, expect, it } from "vitest";

import type { PlacementInput } from "../presentation/canvas/selection-surface-geometry";
import { placeSelectionSurface } from "../presentation/canvas/selection-surface-placement";

const input: PlacementInput = {
  selection: { left: 220, top: 360, right: 380, bottom: 560 },
  members: [],
  viewport: { width: 900, height: 700 },
  insets: { top: 8, right: 16, bottom: 8, left: 16 },
  reserved: [{ left: 350, top: 8, right: 650, bottom: 90 }],
  surface: { width: 260, height: 620 },
  kind: "inspector",
  gap: 12,
};

describe("canvas inspector placement", () => {
  it("keeps a tall editor beside the selection and scrolls below fixed chrome", () => {
    expect(placeSelectionSurface(input)).toEqual({
      left: 392,
      top: 90,
      maxHeight: 602,
      side: "right",
    });
  });

  it("returns an editor from an old above position to the available side", () => {
    expect(placeSelectionSurface({ ...input, previousSide: "above" })?.side).toBe("right");
  });

  it("keeps a compact edit toolbar 12px outside the selected image", () => {
    expect(
      placeSelectionSurface({
        ...input,
        kind: "compact-editor",
        surface: { width: 180, height: 36 },
      }),
    ).toEqual({
      left: 210,
      top: 312,
      maxHeight: 36,
      side: "above",
    });
  });

  it("moves a compact edit toolbar beside a viewport-filling image", () => {
    expect(
      placeSelectionSurface({
        ...input,
        kind: "compact-editor",
        selection: { left: 220, top: 12, right: 380, bottom: 688 },
        surface: { width: 180, height: 36 },
      })?.side,
    ).toBe("right");
  });
});
