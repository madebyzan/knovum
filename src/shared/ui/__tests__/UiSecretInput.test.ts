import { mount } from "@vue/test-utils";
import { describe, expect, it, onTestFinished } from "vitest";
import { createI18n } from "vue-i18n";

import UiSecretInput from "../UiSecretInput.vue";

const i18n = createI18n({
  legacy: false,
  locale: "en",
  messages: { en: { common: { showSecret: "Show secret", hideSecret: "Hide secret" } } },
});

function render(props: { modelValue?: string; placeholder?: string } = {}) {
  const wrapper = mount(UiSecretInput, {
    props: { modelValue: "", ...props },
    attachTo: document.body,
    global: { plugins: [i18n] },
  });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}

describe("UiSecretInput", () => {
  it("reveals the entire typed value, including text added after revealing", async () => {
    const wrapper = render();
    const input = wrapper.get("input");
    await input.setValue("abc");
    await wrapper.get('button[aria-label="Show secret"]').trigger("click");
    expect(input.attributes("type")).toBe("text");
    await input.setValue("abcdef");
    expect((input.element as HTMLInputElement).value).toBe("abcdef");
    expect((input.element as HTMLInputElement).value).not.toContain("•");
  });

  it("shows a loaded value masked and reveals all of it", async () => {
    const wrapper = render({ modelValue: "stored-key" });
    const input = wrapper.get("input");
    expect((input.element as HTMLInputElement).value).toBe("stored-key");
    expect(input.attributes("type")).toBe("password");
    await wrapper.get('button[aria-label="Show secret"]').trigger("click");
    expect(input.attributes("type")).toBe("text");
    expect((input.element as HTMLInputElement).value).toBe("stored-key");
  });

  it("hides the reveal control while empty and shows it once text exists", async () => {
    const wrapper = render();
    expect(wrapper.find('button[aria-label="Show secret"]').exists()).toBe(false);
    await wrapper.get("input").setValue("x");
    expect(wrapper.find('button[aria-label="Show secret"]').exists()).toBe(true);
  });

  it("commits on blur only after the value was edited", async () => {
    const wrapper = render({ modelValue: "stored-key" });
    const input = wrapper.get("input");
    await input.trigger("focus");
    await input.trigger("blur");
    expect(wrapper.emitted("commit")).toBeUndefined();
    await input.setValue("changed");
    await input.trigger("blur");
    expect(wrapper.emitted("commit")).toHaveLength(1);
  });
});
