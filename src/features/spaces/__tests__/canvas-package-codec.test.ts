import { describe, expect, it } from "vitest";

import { canvasPackageCodec } from "../canvas/canvas-package-codec";

describe("Canvas package codec", () => {
  it("remaps node references in the neutral document", () => {
    const state = canvasPackageCodec.remap(
      {
        document: {
          version: 1,
          viewport: { x: 0, y: 0, zoom: 1 },
          nodes: [
            {
              nodeId: "before-parent",
              frame: { x: 0, y: 0, width: 640, height: 480, rotation: 0 },
              sizeAuthority: "frame",
              zIndex: 0,
            },
            {
              nodeId: "before-child",
              frame: { x: 20, y: 20, width: 320, height: 180, rotation: 0 },
              sizeAuthority: "frame",
              zIndex: 1,
            },
          ],
        },
      },
      {
        nodeIds: new Map([
          ["before-parent", "after-parent"],
          ["before-child", "after-child"],
        ]),
      },
    );

    expect(state.document.nodes.map(({ nodeId }) => nodeId)).toEqual(["after-parent", "after-child"]);
  });
});
