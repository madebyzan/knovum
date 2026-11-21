import { describe, expect, it } from "vitest";

import { mergeLineRects, normalizeBoxToPage } from "../pdf-rect-geometry";

describe("pdf-rect-geometry", () => {
  it("normalizes client bounding boxes into clamped [0..1] page coordinates", () => {
    const pageBox = { left: 100, top: 50, right: 600, bottom: 1050, width: 500, height: 1000 };
    const rect = normalizeBoxToPage(
      { left: 150, top: 250, right: 400, bottom: 280, width: 250, height: 30 },
      pageBox,
    );
    expect(rect).toEqual({ x: 0.1, y: 0.2, width: 0.5, height: 0.03 });
    expect(
      normalizeBoxToPage(
        { left: 100, top: 50, right: 100.5, bottom: 50.5, width: 0.5, height: 0.5 },
        pageBox,
      ),
    ).toBeUndefined();
  });

  it("merges adjacent same-line text rects while keeping separate lines distinct", () => {
    const merged = mergeLineRects([
      { x: 0.1, y: 0.2, width: 0.2, height: 0.02 },
      { x: 0.305, y: 0.201, width: 0.15, height: 0.02 },
      { x: 0.1, y: 0.25, width: 0.3, height: 0.02 },
    ]);
    expect(merged).toEqual([
      { x: 0.1, y: 0.2, width: 0.355, height: 0.021 },
      { x: 0.1, y: 0.25, width: 0.3, height: 0.02 },
    ]);
  });
});
