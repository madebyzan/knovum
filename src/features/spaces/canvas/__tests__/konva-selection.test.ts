import { describe, expect, it, vi } from "vitest";
import { nextTick, shallowRef } from "vue";

import type { CanvasSelectionSnapshot } from "../contracts/canvas-editor-port";
import { useCanvasSelection } from "../konva/selection/use-canvas-selection";
import type { CanvasNodeLayoutV1 } from "../schema/document";

const layouts: CanvasNodeLayoutV1[] = [
  {
    nodeId: "a",
    frame: { x: 0, y: 0, width: 100, height: 80, rotation: 0 },
    sizingMode: "frame",
    zIndex: 0,
  },
  {
    nodeId: "b",
    frame: { x: 140, y: 0, width: 100, height: 80, rotation: 0 },
    sizingMode: "frame",
    zIndex: 1,
  },
];

describe("Konva canvas selection", () => {
  it("preserves mixed selection roots and reports all members when a selected container is pressed", () => {
    const changed = vi.fn<(selection: CanvasSelectionSnapshot) => void>();
    const selection = useCanvasSelection({
      availableStructureNodeIds: shallowRef(new Set(["section", "group"])),
      changed,
      layouts: shallowRef(layouts),
      relationBoundsById: shallowRef(new Map()),
      structureBoundsById: shallowRef(
        new Map([
          ["section", { x: 300, y: 0, width: 400, height: 300 }],
          ["group", { x: 800, y: 0, width: 100, height: 80 }],
        ]),
      ),
      structureSelectionForNode: () => undefined,
    });
    selection.selectNode("a", pointer());
    selection.selectStructureNode("section", pointer({ shiftKey: true }));
    selection.selectStructureNode("group", pointer({ shiftKey: true }));
    selection.selectStructureNode("section", pointer());
    expect(selection.structureNodeIds.value).toEqual(new Set(["section", "group"]));
    expect(selection.nodeIds.value).toEqual(new Set(["a"]));
    expect(changed).toHaveBeenLastCalledWith(
      expect.objectContaining({
        bounds: { x: 0, y: 0, width: 900, height: 300 },
        members: [
          layouts[0]!.frame,
          { x: 300, y: 0, width: 400, height: 300 },
          { x: 800, y: 0, width: 100, height: 80 },
        ],
      }),
    );
  });

  it("preserves a multi-selection when an already-selected node begins a drag", () => {
    const changed = vi.fn<(selection: CanvasSelectionSnapshot) => void>();
    const selection = useCanvasSelection({
      availableStructureNodeIds: shallowRef(new Set()),
      changed,
      layouts: shallowRef(layouts),
      relationBoundsById: shallowRef(new Map()),
      structureBoundsById: shallowRef(new Map()),
      structureSelectionForNode: () => undefined,
    });

    selection.selectNode("a", pointer());
    selection.selectNode("b", pointer({ shiftKey: true }));
    selection.selectNode("b", pointer());

    expect(selection.nodeIds.value).toEqual(new Set(["a", "b"]));
  });

  it("removes selected nodes that leave the projected scene", async () => {
    const changed = vi.fn<(selection: CanvasSelectionSnapshot) => void>();
    const currentLayouts = shallowRef(layouts);
    const selection = useCanvasSelection({
      availableStructureNodeIds: shallowRef(new Set()),
      changed,
      layouts: currentLayouts,
      relationBoundsById: shallowRef(new Map()),
      structureBoundsById: shallowRef(new Map()),
      structureSelectionForNode: () => undefined,
    });
    selection.setNodeIds(["a", "b"]);

    currentLayouts.value = [layouts[0]!];
    await nextTick();

    expect(selection.nodeIds.value).toEqual(new Set(["a"]));
    expect(changed).toHaveBeenLastCalledWith(expect.objectContaining({ nodeIds: ["a"] }));
  });
});

function pointer(init: PointerEventInit = {}) {
  return new PointerEvent("pointerdown", init);
}
