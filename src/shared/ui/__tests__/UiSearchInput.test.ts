import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import UiSearchInput from "../UiSearchInput.vue";

describe("UiSearchInput", () => {
  it("renders input with placeholder and reflects modelValue", async () => {
    const wrapper = mount(UiSearchInput, {
      props: { modelValue: "zoom", placeholder: "Search shortcuts" },
    });
    const input = wrapper.get("input");
    expect(input.attributes("placeholder")).toBe("Search shortcuts");
    expect((input.element as HTMLInputElement).value).toBe("zoom");
  });

  it("clears value on Escape when non-empty", async () => {
    const wrapper = mount(UiSearchInput, {
      props: { modelValue: "search term" },
    });
    const input = wrapper.get("input");
    await input.trigger("keydown.escape");
    expect(wrapper.emitted("update:modelValue")?.[0]).toEqual([""]);
  });

  it("emits escape when empty and Escape is pressed", async () => {
    const wrapper = mount(UiSearchInput, {
      props: { modelValue: "" },
    });
    const input = wrapper.get("input");
    await input.trigger("keydown.escape");
    expect(wrapper.emitted("escape")).toBeDefined();
  });
});
