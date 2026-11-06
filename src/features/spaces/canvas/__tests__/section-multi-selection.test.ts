import type Konva from "konva";
import { describe, expect, it } from "vitest";
import { shallowRef } from "vue";

import { createStructureIndex } from "../../model/structure/structure-index";
import { arrangementAvailability } from "../../structure/action-availability";
import { captureStructureGesture } from "../konva/interaction/structure-gesture-capture";
import { normalizeCanvasSelection } from "../konva/selection/normalize-selection";
import { useCanvasMarqueeSelection } from "../konva/selection/use-canvas-marquee-selection";

const index = createStructureIndex([
  { id: "first", kind: "section", parentId: null, order: 0, locked: false, title: "First" },
  { id: "second", kind: "section", parentId: null, order: 1, locked: false, title: "Second" },
  { id: "child", kind: "content", nodeId: "picture", parentId: "first", order: 0, locked: false },
]);

describe("section multi selection", () => {
  it("retains sibling sections and removes separately selected descendants", () => {
    const selected = normalizeCanvasSelection(["picture"], ["first", "second"], index);
    expect([...selected.structureNodeIds]).toEqual(["first", "second"]);
    expect([...selected.nodeIds]).toEqual([]);
  });
  it("keeps content selected inside a section when the container was not selected", () => {
    expect([...normalizeCanvasSelection(["picture"], [], index).nodeIds]).toEqual(["picture"]);
  });
  it("captures both sections for a drag and enables the shared arrangement commands", () => {
    const snapshot = {
      editRevision: 0,
      relationStyles: [],
      structure: [...index.structureById.values()],
      layouts: [],
    };
    const frame = { x: 0, y: 0, width: 100, height: 100 };
    const target = { position: () => ({ x: 0, y: 0 }) } as unknown as Konva.Node;
    const capture = captureStructureGesture({
      id: "first",
      target,
      snapshot,
      selectedIds: ["first", "second", "child"],
      boundsByNodeId: new Map([
        ["first", frame],
        ["second", { ...frame, x: 110 }],
        ["child", frame],
      ]),
    });
    expect(capture?.roots).toEqual(["first", "second"]);
    expect(arrangementAvailability(snapshot, ["first", "second"])).toEqual({
      align: true,
      autoLayout: true,
      distribute: false,
    });
  });
  it("selects all fully enclosed sections, including empty sections", () => {
    let point = { x: -10, y: -10 };
    const stage = { getStage: () => stage, getRelativePointerPosition: () => point };
    const structures = shallowRef(new Set<string>());
    const marquee = useCanvasMarqueeSelection({
      clear: () => {},
      notify: () => {},
      layouts: shallowRef([]),
      nodeIds: shallowRef(new Set<string>()),
      structureNodeIds: structures,
      structureSelectionForNode: () => undefined,
      sectionBounds: shallowRef(
        new Map([
          ["first", { x: 0, y: 0, width: 100, height: 100 }],
          ["second", { x: 110, y: 0, width: 100, height: 100 }],
          ["partial", { x: 215, y: 0, width: 100, height: 100 }],
        ]),
      ),
    });
    const event = { target: stage, evt: { button: 0 } } as unknown as Parameters<typeof marquee.begin>[0];
    marquee.begin(event);
    point = { x: 220, y: 120 };
    marquee.move(event);
    marquee.end();
    expect([...structures.value]).toEqual(["first", "second"]);
  });
});
