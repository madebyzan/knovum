import { expect, it } from "vitest";

import { defaultImageEdit } from "../image-editing/image-edit";
import { validateSpaceNodePayload } from "../model/nodes/node-validation";
import { imageSourceNode } from "./support/image-operation-fixtures";

/**
 * A depth field is measurement data rather than a picture. Saving it keeps its
 * own 16-bit bytes, which the engine decides and proves in its own tests.
 */
function depthNode() {
  const node = imageSourceNode();
  return {
    ...node,
    data: {
      ...node.data,
      imageDepth: {
        encoding: "relative-inverse-depth-u16-v1" as const,
        modelSpecId: "depth-spec",
      },
    },
  };
}

it("rejects photo adjustments on a depth analysis field", () => {
  expect(() => validateSpaceNodePayload(depthNode().data)).not.toThrow();
  expect(() => validateSpaceNodePayload({ ...depthNode().data, imageEdit: defaultImageEdit() })).toThrow(
    "Relative depth fields do not support photo edits",
  );
});
