import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import UiNumberField from "../UiNumberField.vue";

describe("UiNumberField", () => {
  it("renders number input with initial modelValue", async () => {
    const wrapper = mount(UiNumberField, {
      props: { modelValue: 42, min: 0, max: 100 },
    });
    const input = wrapper.get("input");
    expect((input.element as HTMLInputElement).value).toBe("42");
    expect(input.attributes("role")).toBe("spinbutton");
  });

  it("applies size class correctly", () => {
    const wrapperSm = mount(UiNumberField, { props: { modelValue: 1, size: "sm" } });
    expect(wrapperSm.get("input").classes()).toContain("h-(--knovum-control-height-sm)");

    const wrapperMd = mount(UiNumberField, { props: { modelValue: 1, size: "md" } });
    expect(wrapperMd.get("input").classes()).toContain("h-(--knovum-control-height-md)");
  });
});
