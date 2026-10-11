import { setActivePinia } from "pinia";
import { expect, it } from "vitest";

import { imageRemovalSize } from "../canvas/presentation/images/image-source-dimensions-store";
import { defaultImageEdit } from "../image-editing/image-edit";
import { imageSourceNode } from "./support/image-operation-fixtures";

it("calculates cropped and rotated removal dimensions without a Pinia context", () => {
  setActivePinia(undefined);
  const node = imageSourceNode();
  node.data.intrinsicWidth = undefined;
  node.data.intrinsicHeight = undefined;
  node.data.imageEdit = {
    ...defaultImageEdit(),
    crop: { x: 0.1, y: 0.2, width: 0.5, height: 0.25 },
    rotation: 90,
  };
  const size = imageRemovalSize(node, { width: 800, height: 600 });
  expect(size?.width).toBeCloseTo(150);
  expect(size?.height).toBeCloseTo(400);
  expect(imageRemovalSize(node, undefined)).toBeUndefined();
});
