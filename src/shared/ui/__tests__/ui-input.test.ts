import { mount } from "@vue/test-utils";
import { expect, it } from "vitest";

import UiInput from "../UiInput.vue";

it("keeps number-input edits within its string model contract", async () => {
  const input = mount(UiInput, { props: { type: "number" } });
  await input.get("input").setValue("4");
  await input.get("input").setValue("");
  expect(input.emitted("update:modelValue")).toEqual([["4"], [""]]);
  input.unmount();
});
