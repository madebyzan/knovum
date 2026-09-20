import { describe, expect, it } from "vitest";

import { canvasPackageCodec } from "../canvas/canvas-package-codec";
import { parseSpaceEditorDocument } from "../canvas/contract/canvas-document";
import { PACKAGE_ASSET_PREFIX } from "../package/contract";

function source() {
  return {
    document: parseSpaceEditorDocument({
      version: 1,
      viewport: { x: 25, y: -10, zoom: 0.8 },
      structure: {
        version: 1,
        items: [
          { kind: "section", id: "s", title: "References", parentId: null, order: 0, locked: false },
          { kind: "group", id: "g", title: "Pair", parentId: "s", order: 0, locked: false },
          { kind: "content", id: "a", nodeId: "a", parentId: "g", order: 0, locked: true },
          { kind: "content", id: "b", nodeId: "b", parentId: "g", order: 1, locked: false },
        ],
      },
      layout: {
        version: 1,
        items: [
          {
            kind: "section",
            itemId: "s",
            position: { x: 100, y: 200 },
            size: { width: 800, height: 600 },
            collapsed: true,
            appearance: { background: `${PACKAGE_ASSET_PREFIX}cover` },
          },
          { kind: "group", itemId: "g", transform: { x: 20, y: 40, scale: 1.5, rotation: 25 } },
          {
            kind: "content",
            itemId: "a",
            transform: { x: 0, y: 0, scale: 1, rotation: 0 },
            size: { width: 120, height: 90 },
            sizeAuthority: "frame",
            fileSubpath: "#page=4",
          },
          {
            kind: "content",
            itemId: "b",
            transform: { x: 180, y: 0, scale: 1, rotation: 0 },
            size: { width: 120, height: 90 },
            sizeAuthority: "content",
          },
        ],
      },
      relationStyles: [{ relationId: "r", color: "blue" }],
    }),
  };
}
function context() {
  return {
    itemIds: new Map(["s", "g", "a", "b"].map((id) => [id, `new-${id}`])),
    nodeIds: new Map(["a", "b"].map((id) => [id, `new-${id}`])),
    relationIds: new Map([["r", "new-r"]]),
    assets: new Map([["cover", "/managed/cover.png"]]),
  };
}

describe("native package structural round trip", () => {
  it("remaps every structural identity while retaining local geometry and presentation", () => {
    const before = source();
    const after = canvasPackageCodec.remap(before, context());
    expect(after.document.structure.items[1]).toMatchObject({ id: "new-g", parentId: "new-s" });
    expect(after.document.structure.items[2]).toMatchObject({
      id: "new-a",
      nodeId: "new-a",
      parentId: "new-g",
      locked: true,
    });
    expect(after.document.layout.items[1]).toEqual({ ...before.document.layout.items[1], itemId: "new-g" });
    expect(after.document.layout.items[2]).toMatchObject({ fileSubpath: "#page=4", sizeAuthority: "frame" });
    expect(after.document.layout.items[0]).toMatchObject({
      collapsed: true,
      appearance: { background: "/managed/cover.png" },
    });
    expect(after.document.viewport).toEqual(before.document.viewport);
    expect(after.document.relationStyles).toEqual([{ relationId: "new-r", color: "blue" }]);
    expect(before).toEqual(source());
  });
  it("rejects missing container or resource mappings instead of dropping data", () => {
    const missingItem = context();
    missingItem.itemIds.delete("g");
    expect(() => canvasPackageCodec.remap(source(), missingItem)).toThrow(/structure item/u);
    const missingAsset = context();
    missingAsset.assets.clear();
    expect(() => canvasPackageCodec.remap(source(), missingAsset)).toThrow(/background asset/u);
  });
  it("rejects unmanaged file references in an external package", () => {
    const value = source();
    const section = value.document.layout.items[0]!;
    if (section.kind !== "section") throw new Error("Invalid fixture");
    section.appearance = { background: "/external/private.png" };
    expect(() => canvasPackageCodec.remap(value, context())).toThrow(/packaged resources/u);
  });
});
