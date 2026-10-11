import { describe, expect, it } from "vitest";

import { validateSpaceNodePayload } from "../../model/nodes/node-validation";
import { centeredCrop, defaultImageEdit, nextQuarterTurn } from "../image-edit";

describe("canvas image editing", () => {
  it("creates centered physical-aspect crops", () => {
    expect(centeredCrop(2, 1)).toEqual({ x: 0.25, y: 0, width: 0.5, height: 1 });
    expect(centeredCrop(0.5, 1)).toEqual({ x: 0, y: 0.25, width: 1, height: 0.5 });
  });

  it("cycles quarter-turn rotations", () => {
    expect(nextQuarterTurn(0)).toBe(90);
    expect(nextQuarterTurn(270)).toBe(0);
  });

  it("keeps derived mask strokes out of original-image edits", () => {
    const edit = defaultImageEdit();
    expect(edit.version).toBe(1);
    expect(() =>
      validateSpaceNodePayload({
        kind: "image",
        title: "Image",
        fileName: "image.png",
        assetPath: "/managed/image.png",
        imageEdit: { ...edit, eraserStrokes: [] },
      }),
    ).toThrow(/eraserStrokes/);
    for (const version of [0, 2, 5, 6]) {
      expect(() =>
        validateSpaceNodePayload({
          kind: "image",
          title: "Image",
          fileName: "image.png",
          assetPath: "/managed/image.png",
          imageEdit: { ...edit, version },
        }),
      ).toThrow(/version/);
    }
    expect(() =>
      validateSpaceNodePayload({
        kind: "image",
        title: "Image",
        fileName: "image.png",
        assetPath: "/managed/image.png",
        imageEdit: { ...edit, crop: { x: 0.8, y: 0, width: 0.5, height: 1 } },
      }),
    ).toThrow("Crop exceeds the image width");
  });
});
