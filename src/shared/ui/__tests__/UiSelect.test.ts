import { mount } from "@vue/test-utils";
import { expect, it, vi } from "vitest";

import UiSelect from "../UiSelect.vue";

it("forwards interaction attributes and focus handlers to its combobox trigger", async () => {
  const onFocus = vi.fn<(event: FocusEvent) => void>();
  const wrapper = mount(UiSelect, {
    props: { modelValue: "git", label: "Source", options: [{ value: "git", label: "Git" }] },
    attrs: { "data-testid": "source", "aria-describedby": "source-help", onFocus },
  });
  const trigger = wrapper.get('[role="combobox"]');
  expect(trigger.attributes("data-testid")).toBe("source");
  expect(trigger.attributes("aria-describedby")).toBe("source-help");
  expect(trigger.attributes("aria-label")).toBe("Source");
  await trigger.trigger("focus");
  expect(onFocus).toHaveBeenCalledOnce();
  wrapper.unmount();
});
