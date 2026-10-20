import { mount } from "@vue/test-utils";
import { expect, it } from "vitest";
import { defineComponent, nextTick, shallowReactive } from "vue";

import { useCanvasTextMeasurement } from "../use-canvas-text-measurement";

it("reuses canonical measurements across remounts and invalidates changed geometry or content", async () => {
  const fonts = new EventTarget();
  Object.defineProperty(document, "fonts", { value: fonts, configurable: true });
  const metrics = shallowReactive({
    content: { type: "doc" },
    width: 200,
    fontSize: 14,
    scale: 1,
    alignment: "left",
  });
  let measurement!: ReturnType<typeof useCanvasTextMeasurement>;
  const component = defineComponent({
    setup() {
      measurement = useCanvasTextMeasurement(() => metrics);
      return () => null;
    },
  });
  const first = mount(component);
  expect(measurement.required.value).toBe(true);
  measurement.record(140);
  first.unmount();
  const second = mount(component);
  expect(measurement.cachedHeight.value).toBe(140);
  metrics.width = 240;
  await nextTick();
  expect(measurement.required.value).toBe(true);
  measurement.record(120);
  metrics.scale = 2;
  await nextTick();
  expect(measurement.required.value).toBe(true);
  measurement.record(240);
  fonts.dispatchEvent(new Event("loadingdone"));
  await nextTick();
  expect(measurement.required.value).toBe(true);
  measurement.record(240);
  metrics.content = { type: "doc" };
  await nextTick();
  expect(measurement.required.value).toBe(true);
  second.unmount();
});
