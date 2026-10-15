import { mount } from "@vue/test-utils";
import type Konva from "konva";
import { Konva as konvaGlobal } from "konva/lib/Global";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { defineComponent, effectScope, shallowRef, useTemplateRef } from "vue";

import { createCanvasCamera } from "../konva/viewport/canvas-camera";
import { useCameraStage } from "../konva/viewport/use-camera-stage";
import { useCameraStyle } from "../konva/viewport/use-camera-style";
import { useTestAnimationFrames } from "./support/animation-frames";

/** Konva stages need a real 2D context, so the projection is verified against the stage contract. */
function stageDouble(layerCount: number) {
  const layers = Array.from({ length: layerCount }, () => ({ draw: vi.fn<() => void>() }));
  const position = vi.fn<(value: { x: number; y: number }) => void>();
  const scale = vi.fn<(value: { x: number; y: number }) => void>();
  const stage = { getLayers: () => layers, position, scale } as unknown as Konva.Stage;
  return { drawCounts: () => layers.map(({ draw }) => draw.mock.calls.length), position, scale, stage };
}

describe("camera projection", () => {
  let runFrames: () => void;

  beforeEach(() => (runFrames = useTestAnimationFrames().run));
  afterEach(() => {
    vi.unstubAllGlobals();
    konvaGlobal.autoDrawEnabled = true;
  });

  it("owns the render loop while moving and draws every layer once per frame", () => {
    const { drawCounts, position, scale, stage } = stageDouble(3);
    const camera = createCanvasCamera({ x: 0, y: 0, zoom: 1 });
    const scope = effectScope();
    scope.run(() => useCameraStage(camera, shallowRef(stage)));
    expect(drawCounts()).toEqual([1, 1, 1]);

    camera.set({ x: -60, y: -20, zoom: 1 });
    camera.set({ x: -120, y: -40, zoom: 1 });
    runFrames();
    expect(position).toHaveBeenLastCalledWith({ x: -120, y: -40 });
    expect(drawCounts()).toEqual([2, 2, 2]);
    expect(konvaGlobal.autoDrawEnabled).toBe(false);

    runFrames();
    expect(konvaGlobal.autoDrawEnabled).toBe(true);
    expect(drawCounts()).toEqual([3, 3, 3]);
    expect(scale).toHaveBeenLastCalledWith({ x: 1, y: 1 });
    scope.stop();
  });

  it("restores Konva drawing when the surface is discarded mid-gesture", () => {
    const { scale, stage } = stageDouble(1);
    const camera = createCanvasCamera({ x: 0, y: 0, zoom: 1 });
    const scope = effectScope();
    scope.run(() => useCameraStage(camera, shallowRef(stage)));
    camera.set({ x: 10, y: 0, zoom: 2 });
    runFrames();
    expect(konvaGlobal.autoDrawEnabled).toBe(false);
    expect(scale).toHaveBeenLastCalledWith({ x: 2, y: 2 });
    scope.stop();
    expect(konvaGlobal.autoDrawEnabled).toBe(true);
  });

  it("moves a camera-anchored element without rendering its component again", async () => {
    const camera = createCanvasCamera({ x: 0, y: 0, zoom: 1 });
    let renders = 0;
    const wrapper = mount(
      defineComponent({
        setup() {
          const world = useTemplateRef<HTMLElement>("world");
          useCameraStyle(camera, world, ({ x, y, zoom }) => ({
            transform: `translate(${x}px, ${y}px) scale(${zoom})`,
          }));
          return { countRender: () => (renders += 1) };
        },
        template: `<div ref="world" :data-renders="countRender()" />`,
      }),
    );
    await wrapper.vm.$nextTick();
    expect(renders).toBe(1);

    camera.set({ x: -300, y: 25, zoom: 1 });
    runFrames();
    await wrapper.vm.$nextTick();
    expect(wrapper.element.getAttribute("style")).toContain("translate(-300px, 25px)");
    expect(renders).toBe(1);
    wrapper.unmount();
  });
});
