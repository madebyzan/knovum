import type { Editor } from "@tiptap/vue-3";
import { shallowMount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import { i18n } from "../../../../foundation/i18n/instance";
import { CANVAS_TEXT_SURFACE_ATTRIBUTE } from "../presentation/nodes/canvas-text-surface";
import SpaceCanvasTextSelectionBar from "../presentation/nodes/SpaceCanvasTextSelectionBar.vue";

describe("canvas text selection bar", () => {
  it("makes the floating menu root, which WebKit focuses on a press, the editing surface of its node", () => {
    const editor = {
      getAttributes: () => ({}),
      isActive: () => false,
      on: () => undefined,
      off: () => undefined,
    } as unknown as Editor;
    const wrapper = shallowMount(SpaceCanvasTextSelectionBar, {
      props: {
        nodeId: "text-node",
        kind: "text",
        editor,
        defaultFontSize: 18,
        defaultAlignment: "left",
        typographyScale: 1,
      },
      global: { plugins: [i18n], stubs: { BubbleMenu: { template: "<div><slot /></div>" } } },
    });

    expect(wrapper.element.firstElementChild?.getAttribute(CANVAS_TEXT_SURFACE_ATTRIBUTE)).toBe("text-node");
    wrapper.unmount();
  });
  it.each(["note", "text"] as const)("exposes text color only for text nodes (%s)", (kind) => {
    const editor = {
      getAttributes: () => ({}),
      isActive: () => false,
      on: () => undefined,
      off: () => undefined,
    } as unknown as Editor;
    const wrapper = shallowMount(SpaceCanvasTextSelectionBar, {
      props: {
        nodeId: "text-node",
        kind,
        editor,
        defaultFontSize: 18,
        defaultAlignment: "left",
        typographyScale: 1,
      },
      global: { plugins: [i18n], stubs: { BubbleMenu: { template: "<div><slot /></div>" } } },
    });
    expect(wrapper.findComponent({ name: "SpaceCanvasColorSwatches" }).exists()).toBe(kind === "text");
    wrapper.unmount();
  });
});
