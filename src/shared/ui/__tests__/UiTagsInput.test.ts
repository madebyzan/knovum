import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import UiTagsInput from "../UiTagsInput.vue";

describe("UiTagsInput", () => {
  it("renders existing tags", () => {
    const wrapper = mount(UiTagsInput, {
      props: { modelValue: ["design", "system"] },
    });
    const items = wrapper.findAll("[data-reka-collection-item]");
    expect(items.length).toBe(2);
    expect(wrapper.text()).toContain("design");
    expect(wrapper.text()).toContain("system");
  });

  it("renders placeholder when empty", () => {
    const wrapper = mount(UiTagsInput, {
      props: { modelValue: [], placeholder: "Add tags..." },
    });
    expect(wrapper.get("input").attributes("placeholder")).toBe("Add tags...");
  });
});
