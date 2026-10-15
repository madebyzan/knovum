import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { effectScope, nextTick, reactive, shallowRef } from "vue";

import type { CanvasViewport } from "../contracts/canvas-editor-port";
import { createCanvasCamera } from "../konva/viewport/canvas-camera";
import { useCameraHostAnchor } from "../konva/viewport/use-camera-host-anchor";
import { useTestAnimationFrames } from "./support/animation-frames";

const SETTLE_MS = 160;

function hostDouble(rect: { left: number; top: number }) {
  return { getBoundingClientRect: () => ({ ...rect }) } as unknown as HTMLElement;
}

function anchored(rect: { left: number; top: number }, camera = createCanvasCamera({ x: 0, y: 0, zoom: 1 })) {
  const host = shallowRef<HTMLElement | null>(hostDouble(rect));
  const content = shallowRef<HTMLElement | null>(document.createElement("div"));
  const size = reactive({ height: 800, width: 1000 });
  const projected: CanvasViewport[] = [];
  camera.observeFrames((value) => projected.push(value));
  const scope = effectScope();
  scope.run(() => useCameraHostAnchor(camera, host, content, size));
  return { camera, content, projected, scope, size };
}

describe("camera host anchor", () => {
  let frames: ReturnType<typeof useTestAnimationFrames>;

  beforeEach(() => {
    frames = useTestAnimationFrames();
    vi.useFakeTimers();
  });
  afterEach(() => {
    vi.useRealTimers();
    vi.unstubAllGlobals();
  });

  it("compensates a leading-edge shift on the content layer, then adopts it once in the camera", async () => {
    const rect = { left: 0, top: 0 };
    const { camera, content, projected, scope, size } = anchored(rect);
    await nextTick();

    // A sidebar takes 280px off the leading edge over several frames of its transition.
    const frame = async (left: number) => {
      rect.left = left;
      size.width = 1000 - left;
      await nextTick();
    };
    await frame(100);
    await frame(200);
    await frame(280);

    // Compensated by the compositor: no camera write, so no layer is redrawn per frame.
    expect(content.value?.style.transform).toBe("translate3d(-280px, 0px, 0)");
    expect(camera.get()).toEqual({ x: 0, y: 0, zoom: 1 });
    expect(projected).toEqual([]);

    vi.advanceTimersByTime(SETTLE_MS);

    // Adopted in one frame: the transform is released as the camera takes the shift over.
    expect(content.value?.style.transform).toBe("");
    expect(camera.get()).toEqual({ x: -280, y: 0, zoom: 1 });
    expect(projected).toEqual([{ x: -280, y: 0, zoom: 1 }]);
    scope.stop();
  });

  it("leaves content and camera untouched when only the trailing edge moves", async () => {
    const rect = { left: 0, top: 0 };
    const { camera, content, projected, scope, size } = anchored(
      rect,
      createCanvasCamera({ x: 12, y: 34, zoom: 2 }),
    );
    await nextTick();

    size.width = 680;
    await nextTick();
    vi.advanceTimersByTime(SETTLE_MS);

    expect(content.value?.style.transform).toBe("");
    expect(camera.get()).toEqual({ x: 12, y: 34, zoom: 2 });
    expect(projected).toEqual([]);
    expect(frames.pending.size).toBe(0);
    scope.stop();
  });

  it("adopts a growing host in the same frame, since a transform cannot fill uncovered area", async () => {
    const rect = { left: 280, top: 0 };
    const { camera, content, projected, scope, size } = anchored(rect);
    size.width = 720;
    await nextTick();

    rect.left = 180;
    size.width = 820;
    await nextTick();

    expect(content.value?.style.transform).toBe("");
    expect(camera.get()).toEqual({ x: 100, y: 0, zoom: 1 });
    expect(projected).toEqual([{ x: 100, y: 0, zoom: 1 }]);
    scope.stop();
  });

  it("adopts a pending shift when the surface goes away", async () => {
    const rect = { left: 0, top: 0 };
    const { camera, scope, size } = anchored(rect);
    await nextTick();

    rect.left = 64;
    size.width = 936;
    await nextTick();
    expect(camera.get().x).toBe(0);

    scope.stop();

    expect(camera.get()).toEqual({ x: -64, y: 0, zoom: 1 });
  });
});
