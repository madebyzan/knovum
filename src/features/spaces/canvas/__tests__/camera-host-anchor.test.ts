import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { effectScope, nextTick, reactive, shallowRef } from "vue";

import type { CanvasViewport } from "../contracts/canvas-editor-port";
import { createCanvasCamera } from "../konva/viewport/canvas-camera";
import { useCameraHostAnchor } from "../konva/viewport/use-camera-host-anchor";
import { useTestAnimationFrames } from "./support/animation-frames";

function hostDouble(rect: { left: number; top: number }) {
  return { getBoundingClientRect: () => ({ ...rect }) } as unknown as HTMLElement;
}

function anchored(rect: { left: number; top: number }, camera = createCanvasCamera({ x: 0, y: 0, zoom: 1 })) {
  const host = shallowRef<HTMLElement | null>(hostDouble(rect));
  const size = reactive({ height: 800, width: 1000 });
  const projected: CanvasViewport[] = [];
  camera.observeFrames((value) => projected.push(value));
  const scope = effectScope();
  scope.run(() => useCameraHostAnchor(camera, host, size));
  return { camera, projected, scope, size };
}

describe("camera host anchor", () => {
  let frames: ReturnType<typeof useTestAnimationFrames>;

  beforeEach(() => {
    frames = useTestAnimationFrames();
  });
  afterEach(() => vi.unstubAllGlobals());

  it("folds a leading-edge shift into the camera within the same frame", async () => {
    const rect = { left: 0, top: 0 };
    const { camera, projected, scope, size } = anchored(rect);
    await nextTick();

    // A sidebar takes 280px off the leading edge in one step.
    rect.left = 280;
    size.width = 720;
    await nextTick();

    expect(camera.get()).toEqual({ x: -280, y: 0, zoom: 1 });
    expect(projected).toEqual([{ x: -280, y: 0, zoom: 1 }]);
    scope.stop();
  });

  it("leaves the camera alone when only the trailing edge moves", async () => {
    const rect = { left: 0, top: 0 };
    const { camera, projected, scope, size } = anchored(rect, createCanvasCamera({ x: 12, y: 34, zoom: 2 }));
    await nextTick();

    size.width = 680;
    await nextTick();

    expect(camera.get()).toEqual({ x: 12, y: 34, zoom: 2 });
    expect(projected).toEqual([]);
    expect(frames.pending.size).toBe(0);
    scope.stop();
  });

  it("compensates a growing host too, so uncovered content is painted", async () => {
    const rect = { left: 280, top: 0 };
    const { camera, projected, scope, size } = anchored(rect);
    size.width = 720;
    await nextTick();

    rect.left = 180;
    size.width = 820;
    await nextTick();

    expect(camera.get()).toEqual({ x: 100, y: 0, zoom: 1 });
    expect(projected).toEqual([{ x: 100, y: 0, zoom: 1 }]);
    scope.stop();
  });
});
