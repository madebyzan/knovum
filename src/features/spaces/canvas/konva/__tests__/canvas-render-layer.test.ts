import { mount } from "@vue/test-utils";
import type Konva from "konva";
import { Konva as konvaGlobal } from "konva/lib/Global";
import { Layer } from "konva/lib/Layer";
import { afterEach, describe, expect, it, vi } from "vitest";
import { defineComponent, effectScope, shallowRef } from "vue";
import { Stage as KonvaStage } from "vue-konva/core";

import { CanvasRenderLayer, KonvaCanvasLayer } from "../rendering/canvas-render-layer";
import { createCanvasCamera } from "../viewport/canvas-camera";
import { useCameraStage } from "../viewport/use-camera-stage";
import { useTestAnimationFrames } from "./support/animation-frames";

describe("canvas render ownership", () => {
  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    konvaGlobal.autoDrawEnabled = true;
  });

  it("gives explicit vendor batchDraw calls to the camera owner until settlement", () => {
    const scheduled = vi.spyOn(Layer.prototype, "batchDraw").mockReturnThis();
    // No canvas context is needed to test scheduling ownership.
    const layer = Object.create(CanvasRenderLayer.prototype) as CanvasRenderLayer;
    layer.setCameraOwned(false);
    expect(layer.batchDraw()).toBe(layer);
    expect(scheduled).toHaveBeenCalledTimes(1);

    layer.setCameraOwned(true);
    expect(layer.batchDraw()).toBe(layer);
    expect(scheduled).toHaveBeenCalledTimes(1);

    layer.setCameraOwned(false);
    layer.batchDraw();
    expect(scheduled).toHaveBeenCalledTimes(2);
  });
  it("engages layer ownership before stage changes and releases it on settlement", () => {
    const frames = useTestAnimationFrames();
    const scheduled = vi.spyOn(Layer.prototype, "batchDraw").mockReturnThis();
    const layer = Object.create(CanvasRenderLayer.prototype) as CanvasRenderLayer;
    layer.setCameraOwned(false);
    const draw = vi.fn<() => CanvasRenderLayer>().mockReturnValue(layer);
    layer.draw = draw;
    layer.drawScene = draw;
    const stage = {
      getLayers: () => [layer],
      position: () => layer.batchDraw(),
      scale: () => layer.batchDraw(),
      listening: vi.fn<(value: boolean) => void>(),
    } as unknown as Konva.Stage;
    const camera = createCanvasCamera({ x: 0, y: 0, zoom: 1 });
    const scope = effectScope();
    scope.run(() => useCameraStage(camera, shallowRef(stage), shallowRef(false)));
    scheduled.mockClear();
    draw.mockClear();
    camera.set({ x: 20, y: 0, zoom: 1 });
    frames.run();
    expect(scheduled).not.toHaveBeenCalled();
    expect(draw).toHaveBeenCalledTimes(1);
    frames.run();
    expect(scheduled).toHaveBeenCalled();
    scope.stop();
  });

  it("mounts KonvaCanvasLayer as a CanvasRenderLayer without registering duplicate components on target app", () => {
    const dummyContext = new Proxy({} as CanvasRenderingContext2D, {
      get(_target, prop) {
        if (prop === "canvas") return {};
        return () => {};
      },
      set() {
        return true;
      },
    });
    vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(dummyContext);

    const warnings: string[] = [];
    const wrapper = mount(
      defineComponent({
        components: { KonvaCanvasLayer, KonvaStage },
        template: `
          <KonvaStage :config="{ width: 100, height: 100 }">
            <KonvaCanvasLayer ref="layer" :config="{ id: 'test-layer' }" />
          </KonvaStage>
        `,
      }),
      {
        global: {
          config: {
            warnHandler(msg) {
              warnings.push(msg);
            },
          },
        },
      },
    );

    expect(warnings).toEqual([]);
    const stageVm = wrapper.findComponent(KonvaStage).vm as unknown as { getStage(): Konva.Stage };
    const stage = stageVm.getStage();
    const layers = stage.getLayers();
    expect(layers).toHaveLength(1);
    expect(layers[0]).toBeInstanceOf(CanvasRenderLayer);
    wrapper.unmount();
  });
});
