import { shallowMount } from "@vue/test-utils";
import { Util } from "konva/lib/Util";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { Group as KonvaGroup } from "vue-konva/core";

import { createSpaceNotePayload } from "../../document-editing/tiptap-content";
import CanvasNodeShape from "../konva/rendering/CanvasNodeShape.vue";
import type { CanvasPalette } from "../konva/rendering/use-canvas-palette";
import type { CanvasSceneNode } from "../konva/scene/canvas-scene";
import { createSpaceNode } from "../node";

const sceneNode = {
  node: createSpaceNode(createSpaceNotePayload("Note"), { x: 0, y: 0 }, "note"),
  layout: { frame: { x: 10, y: 20, width: 300, height: 220, rotation: 0 } },
  interaction: { movable: true },
  paintOrder: 0,
} as unknown as CanvasSceneNode;
const palette = { accent: "blue" } as CanvasPalette;

beforeEach(() => vi.spyOn(Util, "isCanvasFarblingActive").mockReturnValue(false));
afterEach(() => vi.restoreAllMocks());

/**
 * Konva node components render no DOM, so Vue cannot inherit listeners passed to them and warns
 * about every one of them. Their events belong in the node configuration instead, which is the
 * single description vue-konva applies to the Konva node.
 */
describe("Konva node event bindings", () => {
  it("describes gestures in the node configuration instead of as vue listeners", () => {
    const warnings: string[] = [];
    const wrapper = shallowMount(CanvasNodeShape, {
      props: { interactive: true, sceneNode, palette, zoom: 1 },
      global: { config: { warnHandler: (message: string) => warnings.push(message) } },
    });
    const config = wrapper.getComponent(KonvaGroup).props("config") as Record<string, unknown>;

    expect(warnings).toEqual([]);
    expect(wrapper.getComponent(KonvaGroup).vm.$.vnode.props).not.toHaveProperty("onDblclick");
    expect(typeof config["onDblclick"]).toBe("function");
    expect(typeof config["onDragstart"]).toBe("function");
    wrapper.unmount();
  });

  it("reports an activation through the configured double click handler", () => {
    const wrapper = shallowMount(CanvasNodeShape, {
      props: { interactive: true, sceneNode, palette, zoom: 1 },
    });
    const config = wrapper.getComponent(KonvaGroup).props("config") as {
      onDblclick(): void;
    };

    config.onDblclick();

    expect(wrapper.emitted("activate")).toEqual([["note"]]);
    wrapper.unmount();
  });

  it("keeps handler identities stable so bindings survive a configuration update", async () => {
    const wrapper = shallowMount(CanvasNodeShape, {
      props: { interactive: true, sceneNode, palette, zoom: 1 },
    });
    const group = wrapper.getComponent(KonvaGroup);
    const before = group.props("config") as Record<string, unknown>;

    await wrapper.setProps({ zoom: 2 });
    const after = group.props("config") as Record<string, unknown>;

    expect(after["onDragstart"]).toBe(before["onDragstart"]);
    wrapper.unmount();
  });
});
