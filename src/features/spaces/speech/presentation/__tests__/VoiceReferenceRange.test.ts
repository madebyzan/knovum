import { flushPromises, mount } from "@vue/test-utils";
import { expect, it, onTestFinished } from "vitest";

import { i18n } from "../../../../../foundation/i18n/instance";
import VoiceReferenceRange from "../VoiceReferenceRange.vue";

it("uses seconds for the displayed interval and accessible keyboard adjustment", async () => {
  const wrapper = mount(VoiceReferenceRange, {
    props: { start: 2, end: 12, durationMs: 20000, disabled: false },
    global: { plugins: [i18n] },
  });
  onTestFinished(() => wrapper.unmount());
  await flushPromises();
  const thumbs = wrapper.findAll('[role="slider"]');
  expect(thumbs[0]!.attributes("aria-valuenow")).toBe("2");
  expect(thumbs[1]!.attributes("aria-valuenow")).toBe("12");
  expect(thumbs[1]!.attributes("aria-valuemax")).toBe("20");
  await thumbs[0]!.trigger("keydown", { key: "ArrowRight" });
  expect(wrapper.emitted("change")?.at(-1)).toEqual([2.1, 12]);
});
