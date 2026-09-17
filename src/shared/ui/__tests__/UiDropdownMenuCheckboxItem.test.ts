import { mount } from "@vue/test-utils";
import { DropdownMenuContent, DropdownMenuRoot } from "reka-ui";
import { describe, expect, it } from "vitest";
import { defineComponent, h } from "vue";

import UiDropdownMenuCheckboxItem from "../UiDropdownMenuCheckboxItem.vue";

const Harness = defineComponent({
  setup(_, { slots }) {
    return () =>
      h(DropdownMenuRoot, { open: true }, () => h(DropdownMenuContent, null, () => slots.default?.()));
  },
});

describe("UiDropdownMenuCheckboxItem", () => {
  it("renders checkbox item with slot content", () => {
    const wrapper = mount(Harness, {
      attachTo: document.body,
      slots: {
        default: () => h(UiDropdownMenuCheckboxItem, { checked: true }, () => "Show Grid"),
      },
    });
    expect(wrapper.text()).toContain("Show Grid");
  });
});
