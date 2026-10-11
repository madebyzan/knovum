import { mount } from "@vue/test-utils";
import type Konva from "konva";
import { expect, it, onTestFinished, vi } from "vitest";
import { defineComponent, shallowRef } from "vue";

import { useStageViewport } from "../canvas/konva/viewport/use-stage-viewport";

it("keeps an empty camera fixed, then accepts navigation when an object exists", () => {
  const enabled = shallowRef(false);
  const update = vi.fn<(value: { x: number; y: number; zoom: number }) => void>();
  let viewport!: ReturnType<typeof useStageViewport>;
  const wrapper = mount(
    defineComponent({
      setup() {
        viewport = useStageViewport({
          host: shallowRef(null),
          initialViewport: { x: 30, y: 40, zoom: 1 },
          navigationEnabled: () => enabled.value,
          update,
        });
        return () => null;
      },
    }),
  );
  onTestFinished(() => wrapper.unmount());
  const wheel = {
    evt: new WheelEvent("wheel", { deltaY: 80, cancelable: true }),
    target: { getStage: () => ({ getPointerPosition: () => ({ x: 100, y: 100 }) }) },
  } as unknown as Konva.KonvaEventObject<WheelEvent>;
  viewport.wheel(wheel);
  viewport.flushViewport();
  expect(wheel.evt.defaultPrevented).toBe(true);
  expect(viewport.getViewport()).toEqual({ x: 30, y: 40, zoom: 1 });
  expect(update).not.toHaveBeenCalled();
  const pointer = {
    ...wheel,
    evt: new PointerEvent("pointerdown", { button: 1, pointerId: 1 }),
  } as unknown as Konva.KonvaEventObject<PointerEvent>;
  expect(viewport.beginPan(pointer)).toBe(false);
  enabled.value = true;
  viewport.wheel(wheel);
  viewport.flushViewport();
  expect(viewport.getViewport()).toEqual({ x: 30, y: -40, zoom: 1 });
  expect(update).toHaveBeenCalledOnce();
  expect(viewport.beginPan(pointer)).toBe(true);
});
