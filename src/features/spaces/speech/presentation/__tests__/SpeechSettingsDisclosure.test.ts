import { mount } from "@vue/test-utils";
import { expect, it, onTestFinished } from "vitest";
import { defineComponent, shallowRef } from "vue";

import SpeechSettingsDisclosure from "../SpeechSettingsDisclosure.vue";

function render(isInvalid = false) {
  const wrapper = mount(SpeechSettingsDisclosure, {
    attachTo: document.body,
    props: { title: "Mix", isInvalid },
    slots: {
      default: defineComponent({
        setup: () => ({ value: shallowRef("0") }),
        template: '<input aria-label="Gain" v-model="value" />',
      }),
    },
  });
  onTestFinished(() => wrapper.unmount());
  return wrapper;
}
it("keeps an edited setting mounted and unchanged when its section is collapsed", async () => {
  const wrapper = render();
  const input = wrapper.get("input");
  expect(input.isVisible()).toBe(false);
  await wrapper.get("button").trigger("click");
  expect(input.isVisible()).toBe(true);
  await input.setValue("-6");
  await wrapper.get("button").trigger("click");
  expect(input.isVisible()).toBe(false);
  expect(input.element.isConnected).toBe(true);
  await wrapper.get("button").trigger("click");
  expect((input.element as HTMLInputElement).value).toBe("-6");
});
it("exposes invalid settings immediately and again if validation fails later", async () => {
  const wrapper = render(true);
  expect(wrapper.get("input").isVisible()).toBe(true);
  await wrapper.setProps({ isInvalid: false });
  await wrapper.get("button").trigger("click");
  expect(wrapper.get("input").isVisible()).toBe(false);
  await wrapper.setProps({ isInvalid: true });
  expect(wrapper.get("input").isVisible()).toBe(true);
  expect(wrapper.get("button").attributes("aria-expanded")).toBe("true");
});
it("renders an optional summary badge only when summary text is provided", async () => {
  const wrapper = render();
  expect(wrapper.get("button").text()).toBe("Mix");
  await wrapper.setProps({ summary: "0 dB" });
  expect(wrapper.get("button").text()).toContain("0 dB");
});
