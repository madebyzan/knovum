import type Konva from "konva";
import { Konva as konvaGlobal } from "konva/lib/Global";
import { Layer } from "konva/lib/Layer";
import { afterEach, describe, expect, it, vi } from "vitest";
import { effectScope, shallowRef } from "vue";

import { CanvasRenderLayer } from "../konva/rendering/canvas-render-layer";
import { createCanvasCamera } from "../konva/viewport/canvas-camera";
import { useCameraStage } from "../konva/viewport/use-camera-stage";
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
});
