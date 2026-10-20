import { describe, expect, it } from "vitest";

import type { PlacementInput } from "../selection-surface-geometry";
import { placeSelectionSurface } from "../selection-surface-placement";

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
  it("keeps the normal action bar above a selection with room for a side inspector", () => {
    expect(
      placeSelectionSurface({ ...input, kind: "actions", gap: 34, surface: { width: 260, height: 42 } }),
    ).toEqual({ left: 170, top: 284, maxHeight: 42, side: "above" });
  });

  it("keeps the normal action bar below a selection near the top edge", () => {
    expect(
      placeSelectionSurface({
        ...input,
        kind: "actions",
        gap: 34,
        surface: { width: 260, height: 42 },
        selection: { left: 220, top: 20, right: 380, bottom: 180 },
      }),
    ).toEqual({ left: 170, top: 214, maxHeight: 42, side: "below" });
  });

  it("keeps the normal action bar on the visible selection edge when both outside edges are full", () => {
    expect(
      placeSelectionSurface({
        ...input,
        kind: "actions",
        gap: 34,
        reserved: [],
        surface: { width: 260, height: 42 },
        selection: { left: 220, top: 12, right: 380, bottom: 688 },
      }),
    ).toEqual({ left: 170, top: 20, maxHeight: 42, side: "inside" });
  });

  it("keeps a tall editor beside the selection and scrolls below fixed chrome", () => {
    expect(placeSelectionSurface(input)).toEqual({
      left: 392,
      top: 90,
      maxHeight: 602,
      side: "right",
    });
  });

  it("gives a newly opened inspector room to grow beyond the old action bar height", () => {
    expect(placeSelectionSurface({ ...input, surface: { width: 260, height: 42 } })).toEqual({
      left: 392,
      top: 439,
      maxHeight: 602,
      side: "right",
    });
  });

  it("returns an editor from an old above position to the available side", () => {
    expect(placeSelectionSurface({ ...input, previousSide: "above" })?.side).toBe("right");
  });

  it("bounds a short inspector above the object by the free band below fixed chrome", () => {
    expect(
      placeSelectionSurface({
        ...input,
        selection: { left: 370, top: 360, right: 530, bottom: 560 },
        surface: { width: 600, height: 42 },
      }),
    ).toEqual({ left: 150, top: 306, maxHeight: 258, side: "above" });
  });

  it("places an overlapping short inspector with room for its editor content", () => {
    expect(
      placeSelectionSurface({
        ...input,
        selection: { left: 100, top: 100, right: 800, bottom: 600 },
        surface: { width: 700, height: 42 },
      }),
    ).toEqual({ left: 100, top: 108, maxHeight: 584, side: "inside" });
  });

  it("shrinks an overlapping editor into the free band below fixed chrome", () => {
    expect(
      placeSelectionSurface({
        ...input,
        selection: { left: 100, top: 100, right: 800, bottom: 600 },
        surface: { width: 700, height: 620 },
      }),
    ).toEqual({ left: 100, top: 90, maxHeight: 602, side: "inside" });
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
