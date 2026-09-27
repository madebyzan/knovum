import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import UiButton from "../UiButton.vue";

describe("UiButton", () => {
  it("keeps disabled behavior on the underlying button", async () => {
    const onClick = vi.fn<(event: MouseEvent) => void>();
    const wrapper = mount(UiButton, {
      props: { disabled: true },
      attrs: { onClick },
      slots: { default: "Save" },
    });
    const button = wrapper.get("button");

    expect(button.attributes("disabled")).toBeDefined();
    await button.trigger("click");
    expect(onClick).not.toHaveBeenCalled();
  });

  it("maps semantic variants without arbitrary styling", () => {
    const wrapper = mount(UiButton, {
      props: { variant: "danger", size: "sm" },
      slots: { default: "Delete" },
    });

    expect(wrapper.get("button").classes()).toEqual(
      expect.arrayContaining(["bg-surface-danger", "text-fg-on-danger", "h-(--knovum-control-height-sm)"]),
    );
  });

  it("keeps subtle danger actions red across interaction states", () => {
    const wrapper = mount(UiButton, {
      props: { variant: "dangerSubtle" },
      slots: { default: "Delete all" },
    });

    expect(wrapper.get("button").classes()).toEqual(
      expect.arrayContaining([
        "bg-surface-danger-subtle",
        "text-fg-danger",
        "hover:text-fg-danger",
        "hover:bg-surface-danger-subtle-hover",
      ]),
    );
  });

  it("renders asChild element when asChild is true", () => {
    const wrapper = mount(UiButton, {
      props: { asChild: true },
      slots: { default: '<a href="/spaces">Navigate</a>' },
    });
    const link = wrapper.get("a");
    expect(link.attributes("href")).toBe("/spaces");
    expect(link.classes()).toContain("inline-flex");
  });

  it.each(["secondary", "ghost"] as const)("uses distinct hover and pressed roles for %s", (variant) => {
    const button = mount(UiButton, { props: { variant } }).get("button");
    expect(button.classes()).toContain("hover:bg-surface-control-hover");
    expect(button.classes()).toContain("active:bg-surface-control-pressed");
    expect(button.classes()).not.toContain("hover:bg-surface-control-pressed");
  });
});
