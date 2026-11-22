import { mount } from "@vue/test-utils";
import { describe, expect, it } from "vitest";
import { createI18n } from "vue-i18n";

import UiCollapsibleClamp from "../UiCollapsibleClamp.vue";

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

function render(props: Record<string, unknown> = {}, content = "Short message") {
  return mount(UiCollapsibleClamp, {
    props,
    slots: { default: `<p>${content}</p>` },
    global: { plugins: [i18n] },
  });
}

async function simulateHeight(wrapper: ReturnType<typeof render>, scrollHeight: number) {
  const inner = wrapper.find("p").element.parentElement!;
  Object.defineProperty(inner, "scrollHeight", { configurable: true, value: scrollHeight });
  wrapper.vm.measure();
  await wrapper.vm.$nextTick();
}

describe("UiCollapsibleClamp", () => {
  it("does not clamp or show a toggle when content fits within maxHeight", async () => {
    const wrapper = render({ maxHeight: 128 });
    await simulateHeight(wrapper, 80);

    expect(wrapper.find("button").exists()).toBe(false);
    expect(wrapper.html()).not.toContain("max-height:");
  });

  it("clamps tall content and toggles expand and collapse on click", async () => {
    const wrapper = render({ maxHeight: 128 }, "Very long user prompt");
    await simulateHeight(wrapper, 320);

    const button = wrapper.get("button");
    expect(button.text()).toContain("展开");
    expect(button.attributes("aria-expanded")).toBe("false");
    expect(wrapper.html()).toContain("max-height: 128px");

    await button.trigger("click");
    expect(button.text()).toContain("收起");
    expect(button.attributes("aria-expanded")).toBe("true");
    expect(wrapper.html()).not.toContain("max-height: 128px");

    await button.trigger("click");
    expect(button.text()).toContain("展开");
    expect(button.attributes("aria-expanded")).toBe("false");
    expect(wrapper.html()).toContain("max-height: 128px");
  });

  it("resets expanded state when contentKey changes", async () => {
    const wrapper = render({ maxHeight: 128, contentKey: "msg-1" }, "Long message 1");
    await simulateHeight(wrapper, 280);

    await wrapper.get("button").trigger("click");
    expect(wrapper.get("button").attributes("aria-expanded")).toBe("true");

    await wrapper.setProps({ contentKey: "msg-2" });
    await simulateHeight(wrapper, 280);
    expect(wrapper.get("button").attributes("aria-expanded")).toBe("false");
  });
});
