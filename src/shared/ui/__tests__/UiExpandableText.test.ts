import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import { createI18n } from "vue-i18n";

import UiExpandableText from "../UiExpandableText.vue";

const i18n = createI18n({
  legacy: false,
  locale: "zh",
  messages: {
    zh: {
      common: {
        expand: "展开",
        collapse: "收起",
      },
    },
  },
});

function render(props: Record<string, unknown> = {}, slots: Record<string, string> = {}) {
  return mount(UiExpandableText, {
    props,
    slots,
    global: { plugins: [i18n] },
  });
}

describe("UiExpandableText", () => {
  it("renders text via prop", () => {
    const wrapper = render({ text: "Hello world text content" });
    expect(wrapper.text()).toContain("Hello world text content");
    expect(wrapper.find("p").exists()).toBe(true);
    expect(wrapper.find("pre").exists()).toBe(false);
  });

  it("renders text via slot", () => {
    const wrapper = render({}, { default: "Slot text content" });
    expect(wrapper.text()).toContain("Slot text content");
  });

  it("renders pre element when preformatted is true", () => {
    const wrapper = render({ text: "Code or OCR block", preformatted: true });
    expect(wrapper.find("pre").exists()).toBe(true);
    expect(wrapper.find("pre").classes()).toContain("whitespace-pre-wrap");
  });

  it("applies clamp class when collapsed and removes it when expanded", async () => {
    const wrapper = render({ text: "Long text", clampLines: 3 });
    const p = wrapper.find("p");
    expect(p.classes()).toContain("line-clamp-3");

    // Manually trigger expand
    wrapper.vm.toggle();
    await wrapper.vm.$nextTick();
    expect(p.classes()).not.toContain("line-clamp-3");
  });

  it("shows expand button when overflow is simulated and toggles state on click", async () => {
    const wrapper = render({ text: "Line 1\nLine 2\nLine 3\nLine 4\nLine 5", clampLines: 2 });
    const el = wrapper.find("p").element;
    Object.defineProperty(el, "scrollHeight", { configurable: true, value: 200 });
    Object.defineProperty(el, "clientHeight", { configurable: true, value: 50 });

    wrapper.vm.checkOverflow();
    await wrapper.vm.$nextTick();

    const button = wrapper.find("button");
    expect(button.exists()).toBe(true);
    expect(button.text()).toContain("展开");
    expect(button.attributes("aria-expanded")).toBe("false");

    await button.trigger("click");
    expect(button.text()).toContain("收起");
    expect(button.attributes("aria-expanded")).toBe("true");
    expect(wrapper.find("p").classes()).not.toContain("line-clamp-2");

    await button.trigger("click");
    expect(button.text()).toContain("展开");
    expect(button.attributes("aria-expanded")).toBe("false");
  });

  it("supports custom expand and collapse labels", async () => {
    const wrapper = render({
      text: "Custom labels test",
      clampLines: 2,
      expandLabel: "查看全部",
      collapseLabel: "隐藏内容",
    });
    const el = wrapper.find("p").element;
    Object.defineProperty(el, "scrollHeight", { configurable: true, value: 200 });
    Object.defineProperty(el, "clientHeight", { configurable: true, value: 50 });

    wrapper.vm.checkOverflow();
    await wrapper.vm.$nextTick();

    const button = wrapper.find("button");
    expect(button.exists()).toBe(true);
    expect(button.text()).toContain("查看全部");

    await button.trigger("click");
    expect(button.text()).toContain("隐藏内容");
  });
});
