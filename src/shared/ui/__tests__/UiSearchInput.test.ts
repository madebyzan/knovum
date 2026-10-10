import { mount } from "@vue/test-utils";
import { describe, expect, it, onTestFinished } from "vitest";
import { createI18n } from "vue-i18n";

import UiSearchInput from "../UiSearchInput.vue";

const i18n = createI18n({
  legacy: false,
  locale: "en",
  messages: { en: { common: { clearSearch: "Clear search" } } },
});

function render(props: { modelValue: string; placeholder?: string; disabled?: boolean }) {
  const wrapper = mount(UiSearchInput, { props, attachTo: document.body, global: { plugins: [i18n] } });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}

describe("UiSearchInput", () => {
  it("renders input with placeholder and reflects modelValue", async () => {
    const wrapper = render({ modelValue: "zoom", placeholder: "Search shortcuts" });
    const input = wrapper.get("input");
    expect(input.attributes("placeholder")).toBe("Search shortcuts");
    expect((input.element as HTMLInputElement).value).toBe("zoom");
  });

  it("clears value on Escape when non-empty", async () => {
    const wrapper = render({ modelValue: "search term" });
    const input = wrapper.get("input");
    await input.trigger("keydown.escape");
    expect(wrapper.emitted("update:modelValue")?.[0]).toEqual([""]);
  });

  it("emits escape when empty and Escape is pressed", async () => {
    const wrapper = render({ modelValue: "" });
    const input = wrapper.get("input");
    await input.trigger("keydown.escape");
    expect(wrapper.emitted("escape")).toBeDefined();
  });
});

it("names the clear control and returns focus to the input after clearing", async () => {
  const wrapper = render({ modelValue: "term" });
  const clear = wrapper.get("button");
  expect(clear.attributes("aria-label")).toBe(i18n.global.t("common.clearSearch"));
  (clear.element as HTMLButtonElement).focus();
  await clear.trigger("click");
  expect(wrapper.emitted("update:modelValue")).toEqual([[""]]);
  expect(document.activeElement).toBe(wrapper.get("input").element);
});
it("does not allow a disabled search to be cleared", async () => {
  const wrapper = render({ modelValue: "term", disabled: true });
  expect(wrapper.get("button").attributes("disabled")).toBeDefined();
  await wrapper.get("button").trigger("click");
  expect(wrapper.emitted("update:modelValue")).toBeUndefined();
});
it("preserves the query when Escape dismisses an input method composition", async () => {
  const wrapper = render({ modelValue: "term" });
  await wrapper.get("input").trigger("keydown.escape", { isComposing: true });
  expect(wrapper.emitted("update:modelValue")).toBeUndefined();
  expect(wrapper.emitted("escape")).toBeUndefined();
});
