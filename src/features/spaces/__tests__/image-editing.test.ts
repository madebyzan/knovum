import { describe, expect, it } from "vitest";

import { centeredCrop, defaultImageEdit, nextQuarterTurn } from "../image-editing/image-edit";
import { validateSpaceNodePayload } from "../model/nodes/schema/node-payload";

describe("canvas image editing", () => {
  it("creates centered physical-aspect crops", () => {
    expect(centeredCrop(2, 1)).toEqual({ x: 0.25, y: 0, width: 0.5, height: 1 });
    expect(centeredCrop(0.5, 1)).toEqual({ x: 0, y: 0.25, width: 1, height: 0.5 });
  });

  it("cycles quarter-turn rotations", () => {
    expect(nextQuarterTurn(0)).toBe(90);
    expect(nextQuarterTurn(270)).toBe(0);
  });

  it("validates normalized eraser strokes and crop bounds", () => {
    const edit = defaultImageEdit();
    const data = validateSpaceNodePayload({
      kind: "image",
      title: "Image",
      fileName: "image.png",
      assetPath: "/managed/image.png",
      imageEdit: { ...edit, eraserStrokes: [{ diameter: 0.05, points: [0.1, 0.2, 0.3, 0.4] }] },
    });
    expect(data.kind === "image" && data.imageEdit?.eraserStrokes).toHaveLength(1);
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
