import type { Editor } from "@tiptap/vue-3";
import { shallowMount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import { i18n } from "../../../../../../foundation/i18n/i18n";
import SpaceCanvasTextSelectionBar from "../SpaceCanvasTextSelectionBar.vue";

describe("canvas text selection bar", () => {
  it("keeps a Vue-owned root while BubbleMenu reparents its own element", () => {
    const editor = {
      getAttributes: () => ({}),
      isActive: () => false,
      on: () => undefined,
      off: () => undefined,
    } as unknown as Editor;
    const wrapper = shallowMount(SpaceCanvasTextSelectionBar, {
      props: { editor, defaultFontSize: 18, defaultAlignment: "left", typographyScale: 1 },
      global: { plugins: [i18n], stubs: { BubbleMenu: true } },
    });

    expect(wrapper.attributes("data-canvas-text-menu-host")).toBeDefined();
    expect(wrapper.findComponent({ name: "BubbleMenu" }).exists()).toBe(true);
  });
});
