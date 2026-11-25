import { BookOpen, LayoutList } from "@lucide/vue";
import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import UiSegmentedControl from "../UiSegmentedControl.vue";

describe("UiSegmentedControl", () => {
  const options = [
    { value: "read", title: "Read", icon: BookOpen },
    { value: "info", title: "Details", icon: LayoutList },
  ] as const;

  it("exposes accessible names and active state in icon-only mode without allowing deselection", async () => {
    const wrapper = mount(UiSegmentedControl, {
      props: {
        modelValue: "read",
        options,
        iconOnly: true,
        ariaLabel: "Reader view",
      },
    });

    const buttons = wrapper.findAll("button");
    expect(buttons).toHaveLength(2);
    expect(buttons[0]?.attributes("aria-label")).toBe("Read");
    expect(buttons[0]?.attributes("aria-pressed")).toBe("true");
    expect(buttons[1]?.attributes("aria-label")).toBe("Details");
    expect(buttons[1]?.attributes("aria-pressed")).toBe("false");

    // Clicking the already-selected option keeps it selected instead of emitting empty string.
    await buttons[0]?.trigger("click");
    expect(wrapper.emitted("update:modelValue")).toBeUndefined();

    await buttons[1]?.trigger("click");
    expect(wrapper.emitted("update:modelValue")).toEqual([["info"]]);
  });
});
