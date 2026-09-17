import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import UiSeparator from "../UiSeparator.vue";

describe("UiSeparator", () => {
  it("renders horizontal separator by default", () => {
    const wrapper = mount(UiSeparator);
    expect(wrapper.get("[role='none'], [role='separator']")).toBeDefined();
    expect(wrapper.classes()).toContain("h-px");
    expect(wrapper.classes()).toContain("w-full");
  });

  it("renders vertical separator when orientation is vertical", () => {
    const wrapper = mount(UiSeparator, { props: { orientation: "vertical" } });
    expect(wrapper.classes()).toContain("w-px");
    expect(wrapper.classes()).toContain("h-3.5");
  });
});
