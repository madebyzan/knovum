import { describe, expect, it } from "vitest";

import type { CanvasDocumentV1 } from "../canvas/contract/canvas-document";
import { rebaseCanvasDocument } from "../canvas/host/canvas-document-rebase";

const base: CanvasDocumentV1 = {
  version: 1,
  viewport: { x: 0, y: 0, zoom: 1 },
  nodes: [],
  relationStyles: [
    { relationId: "a", color: "blue" },
    { relationId: "b", color: "green" },
  ],
};

describe("canvas document rebase", () => {
  it("merges independent relation presentation changes by relation identity", () => {
    const committed: CanvasDocumentV1 = {
      ...base,
      relationStyles: [
        { relationId: "a", color: "purple" },
        { relationId: "b", color: "green" },
      ],
    };
    const pending: CanvasDocumentV1 = {
      ...base,
      relationStyles: [
        { relationId: "a", color: "blue" },
        { relationId: "b", color: "amber" },
      ],
    };

    expect(rebaseCanvasDocument(base, committed, pending).relationStyles).toEqual([
      { relationId: "a", color: "purple" },
      { relationId: "b", color: "amber" },
    ]);
  });
});
