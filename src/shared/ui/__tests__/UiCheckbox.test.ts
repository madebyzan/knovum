import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";

import UiCheckbox from "../UiCheckbox.vue";

describe("UiCheckbox", () => {
  it("renders with label and initial checked state", () => {
    const wrapper = mount(UiCheckbox, {
      props: { modelValue: true, label: "Enable feature" },
    });
    const control = wrapper.get("[role='checkbox']");
    expect(control.attributes("aria-label")).toBe("Enable feature");
    expect(control.attributes("aria-checked")).toBe("true");
  });

  it("does not toggle while disabled", async () => {
    const wrapper = mount(UiCheckbox, {
      props: { modelValue: false, label: "Enable feature", disabled: true },
    });
    await wrapper.get("[role='checkbox']").trigger("click");
    expect(wrapper.emitted("update:modelValue")).toBeUndefined();
  });
});
