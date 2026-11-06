import { describe, expect, it } from "vitest";

import { selectionExportFormats } from "../export/selection-formats";

const offer = (formats: string[], unavailable: string | null = null) => ({ formats, unavailable });

describe("selection export formats", () => {
  it("offers the intersection for homogeneous content, preserving preference order", () => {
    expect(
      selectionExportFormats(["image", "image"], [offer(["png", "jpg", "webp"]), offer(["jpg", "png"])])
        .formats,
    ).toEqual(["default", "png", "jpg"]);
  });
  it("uses individual defaults for mixed content even when extensions overlap", () => {
    expect(selectionExportFormats(["object", "portal"], [offer(["json"]), offer(["json"])]).formats).toEqual([
      "default",
    ]);
  });
  it("never silently omits an unavailable member", () => {
    expect(selectionExportFormats(["image", "image"], [offer(["png"]), offer([], "Preparing")])).toEqual({
      formats: [],
      unavailable: "Preparing",
    });
  });
  it("offers nothing for an empty selection", () => {
    expect(selectionExportFormats([], []).formats).toEqual([]);
  });
});
