import { describe, expect, it, vi } from "vitest";
import { nextTick, shallowRef } from "vue";

import type { CanvasSelectionSnapshot } from "../canvas/contracts/canvas-editor-port";
import { useCanvasSelection } from "../canvas/konva/selection/use-canvas-selection";
import type { CanvasNodeLayoutV1 } from "../canvas/schema/document";

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
  it("preserves a multi-selection when an already-selected node begins a drag", () => {
    const changed = vi.fn<(selection: CanvasSelectionSnapshot) => void>();
    const selection = useCanvasSelection({
      availableStructureNodeIds: shallowRef(new Set()),
      changed,
      layouts: shallowRef(layouts),
      relationBounds: shallowRef(new Map()),
      structureBounds: shallowRef(new Map()),
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
      relationBounds: shallowRef(new Map()),
      structureBounds: shallowRef(new Map()),
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
