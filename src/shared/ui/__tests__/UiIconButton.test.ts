import { mount } from "@vue/test-utils";
import { describe, expect, it, vi } from "vitest";

import UiIconButton from "../UiIconButton.vue";

describe("UiIconButton", () => {
  it("exposes accessible button state without browser title tooltips", () => {
    const wrapper = mount(UiIconButton, {
      props: { label: "Open command palette", active: true },
    });

    const button = wrapper.get("button");
    expect(button.attributes("aria-label")).toBe("Open command palette");
    expect(button.attributes("aria-pressed")).toBe("true");
    expect(button.attributes("title")).toBeUndefined();
  });

  it("forwards trigger attributes and DOM events to the button", async () => {
    const onClick = vi.fn<(event: MouseEvent) => void>();
    const wrapper = mount(UiIconButton, {
      props: { label: "More" },
      attrs: { "aria-expanded": "true", onClick },
    });

    const button = wrapper.get("button");
    expect(button.attributes("aria-expanded")).toBe("true");
    await button.trigger("click");
    expect(onClick).toHaveBeenCalledOnce();
  });
});
