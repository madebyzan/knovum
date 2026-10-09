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

it("does not commit while an IME composition is active", async () => {
  const input = mount(UiInput);

  await input.get("input").trigger("keydown", { key: "Enter", isComposing: true });

  expect(input.emitted("commit")).toBeUndefined();
});

it("does not cancel while an IME composition is active", async () => {
  const input = mount(UiInput);

  await input.get("input").trigger("keydown", { key: "Escape", isComposing: true });

  expect(input.emitted("cancel")).toBeUndefined();
});

it("leaves an unhandled Enter key available to native form submission", () => {
  const input = mount(UiInput);
  const event = new KeyboardEvent("keydown", { key: "Enter", bubbles: true, cancelable: true });

  input.get("input").element.dispatchEvent(event);

  expect(event.defaultPrevented).toBe(false);
});
