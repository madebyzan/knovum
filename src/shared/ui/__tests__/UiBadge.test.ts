import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import UiBadge from "../UiBadge.vue";

describe("UiBadge", () => {
  it("renders default neutral badge with content", () => {
    const wrapper = mount(UiBadge, {
      slots: { default: "Active" },
    });
    expect(wrapper.text()).toBe("Active");
    expect(wrapper.classes()).toContain("bg-surface-raised");
    expect(wrapper.classes()).toContain("text-fg-secondary");
  });

  it("applies accent and danger variants", () => {
    const accentWrapper = mount(UiBadge, { props: { variant: "accent" } });
    expect(accentWrapper.classes()).toContain("text-fg-accent");

    const dangerWrapper = mount(UiBadge, { props: { variant: "danger" } });
    expect(dangerWrapper.classes()).toContain("text-fg-danger");
  });

  it("applies size classes correctly", () => {
    const sm = mount(UiBadge, { props: { size: "sm" } });
    expect(sm.classes()).toContain("h-4");

    const md = mount(UiBadge, { props: { size: "md" } });
    expect(md.classes()).toContain("h-5");
  });
});
