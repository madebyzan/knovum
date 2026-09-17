import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import UiSwitch from "../UiSwitch.vue";

describe("UiSwitch", () => {
  it("exposes a named switch and emits the next checked state", async () => {
    const wrapper = mount(UiSwitch, {
      props: { modelValue: false, label: "Desktop notifications" },
    });
    const control = wrapper.get("[role='switch']");

    expect(control.attributes("aria-label")).toBe("Desktop notifications");
    await control.trigger("click");
    expect(wrapper.emitted("update:modelValue")?.[0]).toEqual([true]);
  });

  it("does not toggle while disabled", async () => {
    const wrapper = mount(UiSwitch, {
      props: { modelValue: false, label: "Desktop notifications", disabled: true },
    });

    await wrapper.get("[role='switch']").trigger("click");
    expect(wrapper.emitted("update:modelValue")).toBeUndefined();
  });
});
