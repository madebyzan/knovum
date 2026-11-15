import { shallowMount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";
import { shallowRef } from "vue";

import CanvasGroupChrome from "../canvas/konva/rendering/CanvasGroupChrome.vue";
import CanvasStructureInteractionLayer from "../canvas/konva/rendering/CanvasStructureInteractionLayer.vue";
import type { CanvasPalette } from "../canvas/konva/rendering/use-canvas-palette";
import { useCanvasMarqueeSelection } from "../canvas/konva/selection/use-canvas-marquee-selection";
import { createStructureIndex } from "../model/structure/structure-index";
import { reconcileGroupScopePath } from "../model/structure/structure-queries";

const index = createStructureIndex([
  { id: "outer", kind: "group", parentId: null, order: 0, locked: false, title: "Outer" },
  { id: "inner", kind: "group", parentId: "outer", order: 0, locked: false, title: "Inner" },
  { id: "child-in", kind: "content", parentId: "inner", order: 0, locked: false, nodeId: "node-in" },
  { id: "child-out", kind: "content", parentId: "outer", order: 1, locked: false, nodeId: "node-outer" },
  { id: "free", kind: "content", parentId: null, order: 1, locked: false, nodeId: "node-free" },
]);

const palette = {
  accent: "#2563eb",
  panel: "#ffffff",
  border: "#e5e7eb",
  borderStrong: "#d1d5db",
  foreground: "#111827",
  foregroundSecondary: "#6b7280",
  typography: { fontFamily: "sans-serif", labelSize: 13, captionSize: 12 },
} as CanvasPalette;

describe("group scope reconciliation", () => {
  it("keeps the scope while selecting descendants inside the scoped group", () => {
    expect(
      reconcileGroupScopePath(["outer", "inner"], index, {
        nodeIds: ["node-in"],
        structureNodeIds: [],
      }),
    ).toEqual(["outer", "inner"]);
  });

  it("narrows nested scope when selecting a sibling inside the parent group", () => {
    expect(
      reconcileGroupScopePath(["outer", "inner"], index, {
        nodeIds: ["node-outer"],
        structureNodeIds: [],
      }),
    ).toEqual(["outer"]);
  });

  it("exits a group scope when selecting the group itself or content outside it", () => {
    expect(
      reconcileGroupScopePath(["outer", "inner"], index, {
        nodeIds: [],
        structureNodeIds: ["inner"],
      }),
    ).toEqual(["outer"]);
    expect(
      reconcileGroupScopePath(["outer"], index, {
        nodeIds: [],
        structureNodeIds: ["outer"],
      }),
    ).toEqual([]);
    expect(
      reconcileGroupScopePath(["outer"], index, {
        nodeIds: ["node-free"],
        structureNodeIds: [],
      }),
    ).toEqual([]);
  });

  it("prunes stale group scopes after dissolution or deletion", () => {
    const dissolved = createStructureIndex([
      { id: "outer", kind: "group", parentId: null, order: 0, locked: false, title: "Outer" },
      { id: "child-in", kind: "content", parentId: "outer", order: 0, locked: false, nodeId: "node-in" },
    ]);
    expect(reconcileGroupScopePath(["outer", "inner"], dissolved)).toEqual(["outer"]);
  });

  it("signals empty stage commit only on completed non-additive clicks with no hits", () => {
    let point = { x: 400, y: 400 };
    const stage = { getStage: () => stage, getRelativePointerPosition: () => point };
    const nodeIds = shallowRef(new Set<string>());
    const structureNodeIds = shallowRef(new Set<string>());
    const onEmptyCommit = vi.fn<() => void>();
    const marquee = useCanvasMarqueeSelection({
      clear: () => {
        nodeIds.value = new Set();
        structureNodeIds.value = new Set();
      },
      onEmptyCommit,
      layouts: shallowRef([
        {
          nodeId: "node-in",
          frame: { x: 0, y: 0, width: 100, height: 80, rotation: 0 },
          sizingMode: "frame",
          zIndex: 0,
        },
      ]),
      nodeIds,
      sectionBounds: shallowRef(new Map()),
      notify: () => {},
      structureNodeIds,
      structureSelectionForNode: () => undefined,
    });
    const plainEvent = (shiftKey = false) =>
      ({
        target: stage,
        evt: { button: 0, shiftKey, metaKey: false, ctrlKey: false },
      }) as unknown as Parameters<typeof marquee.begin>[0];

    // Non-additive empty canvas click -> fires onEmptyCommit
    marquee.begin(plainEvent(false));
    marquee.end();
    expect(onEmptyCommit).toHaveBeenCalledTimes(1);

    // Shift-click empty canvas -> stays in current scope
    marquee.begin(plainEvent(true));
    marquee.end();
    expect(onEmptyCommit).toHaveBeenCalledTimes(1);

    // Dragging marquee over a node inside the group -> keeps group scope
    point = { x: -10, y: -10 };
    marquee.begin(plainEvent(false));
    point = { x: 120, y: 120 };
    marquee.move(plainEvent(false));
    marquee.end();
    expect(nodeIds.value).toEqual(new Set(["node-in"]));
    expect(onEmptyCommit).toHaveBeenCalledTimes(1);

    // Cancelled marquee -> never fires onEmptyCommit
    point = { x: 400, y: 400 };
    marquee.begin(plainEvent(false));
    marquee.cancel();
    marquee.end();
    expect(onEmptyCommit).toHaveBeenCalledTimes(1);
  });

  it("renders group chrome whenever a visible group is selected", () => {
    const wrapper = shallowMount(CanvasStructureInteractionLayer, {
      props: {
        dropPreview: undefined,
        groups: [
          {
            group: { id: "outer", kind: "group", parentId: null, order: 0, locked: false, title: "Outer" },
            bounds: { left: 0, top: 0, right: 200, bottom: 120 },
            sectionId: null,
            visible: true,
            locked: false,
          },
        ],
        interactive: true,
        selectionChromeVisible: true,
        groupTransformEnabled: true,
        palette,
        sections: [],
        selectedIds: new Set(["outer"]),
        zoom: 1,
      },
      global: { renderStubDefaultSlot: true, stubs: { KonvaLayer: true } },
    });
    expect(wrapper.findAllComponents(CanvasGroupChrome)).toHaveLength(1);
    wrapper.unmount();
  });
});
