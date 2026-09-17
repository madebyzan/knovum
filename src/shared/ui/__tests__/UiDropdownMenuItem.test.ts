import { mount } from "@vue/test-utils";
import { DropdownMenuContent, DropdownMenuRoot } from "reka-ui";
import { describe, expect, it } from "vitest";
import { defineComponent, h } from "vue";

import UiDropdownMenuItem from "../UiDropdownMenuItem.vue";

const Harness = defineComponent({
  setup(_, { slots }) {
    return () =>
      h(DropdownMenuRoot, { open: true }, () => h(DropdownMenuContent, null, () => slots.default?.()));
  },
});

describe("UiDropdownMenuItem", () => {
  it("renders slot content and handles destructive class", () => {
    const wrapper = mount(Harness, {
      attachTo: document.body,
      slots: {
        default: () => h(UiDropdownMenuItem, { destructive: true }, () => "Delete file"),
      },
    });
    expect(wrapper.text()).toContain("Delete file");
    expect(wrapper.findComponent(UiDropdownMenuItem).classes()).toContain("text-danger");
  });
});
