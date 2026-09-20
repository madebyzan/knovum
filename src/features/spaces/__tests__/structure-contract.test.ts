import { describe, expect, it } from "vitest";

import { indexStructureSnapshot } from "../organization/structure/structure-context";
import { projectStructure, structureBoundsInParent } from "../organization/structure/structure-projection";
import {
  structureContentIds,
  structureOperationRoots,
  structureSelectionTarget,
} from "../organization/structure/structure-selection";
import { parseStructureSnapshot } from "../organization/structure/structure-snapshot-validation";

const transform = { x: 0, y: 0, rotation: 0, scale: 1 };
function fixture() {
  return {
    structure: {
      version: 1,
      items: [
        { kind: "section", id: "s", parentId: null, order: 0, locked: false, title: "Sources" },
        { kind: "group", id: "g", parentId: "s", order: 0, locked: false, title: "Pair" },
        { kind: "content", id: "a", nodeId: "a", parentId: "g", order: 0, locked: false },
        { kind: "content", id: "b", nodeId: "b", parentId: "g", order: 1, locked: true },
      ],
    },
    layout: {
      version: 1,
      items: [
        {
          kind: "section",
          itemId: "s",
          position: { x: 100, y: 200 },
          size: { width: 400, height: 300 },
          collapsed: false,
        },
        { kind: "group", itemId: "g", transform: { ...transform, rotation: 45, scale: 2 } },
        { kind: "content", itemId: "a", transform, size: { width: 100, height: 80 }, sizeAuthority: "frame" },
        {
          kind: "content",
          itemId: "b",
          transform: { ...transform, x: 120 },
          size: { width: 100, height: 80 },
          sizeAuthority: "frame",
        },
      ],
    },
  };
}
const parse = (value: unknown) => parseStructureSnapshot(value, new Set(["a", "b"]));
describe("Space structure contract", () => {
  it("keeps structure and local layout separate and resolves operation roots", () => {
    const snapshot = parse(fixture());
    const context = indexStructureSnapshot(snapshot);
    expect(structureOperationRoots(context.tree, ["g", "a", "b"]).map((item) => item.id)).toEqual(["g"]);
    expect(structureContentIds(context.tree, ["s", "a"])).toEqual(["a", "b"]);
    expect(structureSelectionTarget(context.tree, "a", null)).toBe("g");
    expect(structureSelectionTarget(context.tree, "a", "g")).toBe("a");
  });
  it("derives tight local Group bounds without rotating world AABBs again", () => {
    const context = indexStructureSnapshot(parse(fixture()));
    const bounds = structureBoundsInParent(context, projectStructure(context), ["g"], "g")!;
    expect(bounds.x).toBeCloseTo(0);
    expect(bounds.y).toBeCloseTo(0);
    expect(bounds.width).toBeCloseTo(220);
    expect(bounds.height).toBeCloseTo(80);
  });
  it("rejects a missing active content placement", () => {
    expect(() => parseStructureSnapshot(fixture(), new Set(["a", "b", "missing"]))).toThrow(/placement/u);
  });
  it("rejects nested Sections", () => {
    const source = fixture();
    source.structure.items[0]!.parentId = "g";
    expect(() => parse(source)).toThrow(/Space root/u);
  });
  it("rejects duplicate sibling orders", () => {
    const source = fixture();
    source.structure.items[3]!.order = 0;
    expect(() => parse(source)).toThrow(/Sibling order/u);
  });
  it("rejects parent cycles without recursive traversal", () => {
    const source = fixture();
    source.structure.items[1]!.parentId = "g";
    expect(() => parse(source)).toThrow(/cycle/u);
  });
  it("rejects kind-mismatched layouts", () => {
    const source = fixture();
    source.layout.items[1]!.itemId = "a";
    expect(() => parse(source)).toThrow(/Layout kind/u);
  });
  it("rejects composed scales outside the contract even when locals are legal", () => {
    const source = fixture();
    source.layout.items[1]!.transform = { ...transform, scale: 1_000_000 };
    source.layout.items[2]!.transform = { ...transform, scale: 2 };
    expect(() => parse(source)).toThrow(/Composed world/u);
  });
});
