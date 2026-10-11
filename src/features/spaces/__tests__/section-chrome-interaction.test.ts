import { shallowMount } from "@vue/test-utils";
import type { Canvas } from "konva/lib/Canvas";
import { HitContext } from "konva/lib/Context";
import { Rect } from "konva/lib/shapes/Rect";
import { Util } from "konva/lib/Util";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { nextTick } from "vue";
import { Rect as KonvaRect, Circle as KonvaCircle } from "vue-konva/core";

import { i18n } from "../../../foundation/i18n/instance";
import type { ProjectedSection } from "../canvas/document/structure-scene";
import CanvasSectionChrome from "../canvas/konva/presentation/CanvasSectionChrome.vue";
import CanvasSectionResizeHandles from "../canvas/konva/presentation/CanvasSectionResizeHandles.vue";
import type { CanvasPalette } from "../canvas/konva/presentation/use-canvas-palette";

const section: ProjectedSection = {
  section: { id: "section", kind: "section", parentId: null, order: 0, locked: false, title: "Sources" },
  layout: {
    structureNodeId: "section",
    kind: "section",
    position: { x: 0, y: 0 },
    size: { width: 400, height: 300 },
    collapsed: false,
  },
  bounds: { left: 0, top: 0, right: 400, bottom: 300 },
  contentCount: 1,
  locked: false,
  temporarilyExpanded: false,
};
const palette = {
  accent: "blue",
  panel: "white",
  border: "gray",
  borderStrong: "black",
  foreground: "black",
  foregroundSecondary: "gray",
  typography: { fontFamily: "sans-serif", labelSize: 13, captionSize: 12 },
} as CanvasPalette;

beforeEach(() => vi.spyOn(Util, "isCanvasFarblingActive").mockReturnValue(false));
afterEach(() => vi.restoreAllMocks());

function renderRects() {
  const wrapper = shallowMount(CanvasSectionChrome, {
    props: { interactive: true, section, selected: true, dropAction: undefined, zoom: 1, palette },
    global: { plugins: [i18n], renderStubDefaultSlot: true },
  });
  return { wrapper, rects: wrapper.findAllComponents(KonvaRect).map((rect) => rect.props("config")!) };
}

describe("Section chrome interaction", () => {
  it("leaves the Section interior out of Konva's hit fill while keeping the edge draggable", () => {
    const { wrapper, rects } = renderRects();
    const border = new Rect(rects.find((rect) => rect.stroke));
    const context = {
      save: vi.fn<() => void>(),
      restore: vi.fn<() => void>(),
      fill: vi.fn<() => void>(),
      stroke: vi.fn<() => void>(),
    };
    const canvas = { ["_canvas"]: { getContext: () => context } } as unknown as Canvas;
    new HitContext(canvas).fillStrokeShape(border);
    expect(context.fill).not.toHaveBeenCalled();
    expect(context.stroke).toHaveBeenCalledOnce();
    wrapper.unmount();
  });

  it("paints the header before the outline so every side retains its full stroke", () => {
    const { wrapper, rects } = renderRects();
    expect(rects.findIndex((rect) => rect.fill === palette.panel)).toBeLessThan(
      rects.findIndex((rect) => rect.stroke),
    );
    wrapper.unmount();
  });

  it("uses diagonal resize cursors and restores the cursor when gestures are disabled", async () => {
    const wrapper = shallowMount(CanvasSectionResizeHandles, {
      props: { interactive: true, section, zoom: 1, palette },
    });
    const container = document.createElement("div");
    let dragging = false;
    const event = {
      target: {
        getStage: () => ({ container: () => container }),
        isDragging: () => dragging,
      },
    };
    const handles = wrapper.findAllComponents(KonvaCircle);
    handles[0]!.vm.$emit("pointerenter", event);
    expect(container.style.cursor).toBe("nwse-resize");
    handles[1]!.vm.$emit("pointerenter", event);
    expect(container.style.cursor).toBe("nesw-resize");
    dragging = true;
    handles[1]!.vm.$emit("pointerleave", event);
    expect(container.style.cursor).toBe("nesw-resize");
    await wrapper.setProps({ interactive: false });
    await nextTick();
    expect(container.style.cursor).toBe("");
    wrapper.unmount();
  });
});
