import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import type { CanvasViewport } from "../contracts/canvas-editor-port";
import { createCanvasCamera } from "../konva/viewport/canvas-camera";
import { useTestAnimationFrames } from "./support/animation-frames";

describe("canvas camera", () => {
  let frames: ReturnType<typeof useTestAnimationFrames>;

  beforeEach(() => (frames = useTestAnimationFrames()));
  afterEach(() => vi.unstubAllGlobals());

  it("projects one frame for a burst of changes and reports settlement once the camera stops", () => {
    const camera = createCanvasCamera({ x: 0, y: 0, zoom: 1 });
    const projected: CanvasViewport[] = [];
    const settled: CanvasViewport[] = [];
    camera.observeFrames((value) => void projected.push(value));
    camera.observeSettled((value) => void settled.push(value));

    for (const x of [10, 20, 30]) camera.set({ x, y: 0, zoom: 1 });
    expect(camera.get()).toEqual({ x: 30, y: 0, zoom: 1 });
    expect(projected).toEqual([]);

    frames.run();
    expect(projected).toEqual([{ x: 30, y: 0, zoom: 1 }]);
    expect(settled).toEqual([]);

    frames.run();
    expect(settled).toEqual([{ x: 30, y: 0, zoom: 1 }]);
    expect(projected).toHaveLength(1);
  });

  it("holds settlement for the length of a gesture and reports it once the hold is released", () => {
    const camera = createCanvasCamera({ x: 0, y: 0, zoom: 1 });
    const settled = vi.fn<(value: CanvasViewport) => void>();
    camera.observeSettled(settled);
    const release = camera.hold();

    camera.set({ x: 40, y: 0, zoom: 1 });
    frames.run();
    frames.run();
    frames.run();
    expect(settled).not.toHaveBeenCalled();

    release();
    release();
    frames.run();
    expect(settled).toHaveBeenCalledTimes(1);
    expect(settled).toHaveBeenLastCalledWith({ x: 40, y: 0, zoom: 1 });

    frames.run();
    expect(settled).toHaveBeenCalledTimes(1);
  });

  it("settles a held camera when the surface forces it", () => {
    const camera = createCanvasCamera({ x: 0, y: 0, zoom: 1 });
    const settled = vi.fn<(value: CanvasViewport) => void>();
    camera.observeSettled(settled);
    camera.hold();

    camera.set({ x: 5, y: 0, zoom: 1 });
    camera.settle();
    expect(settled).toHaveBeenCalledTimes(1);
    expect(frames.pending.size).toBe(0);
  });

  it("defers settlement while the camera keeps moving", () => {
    const camera = createCanvasCamera({ x: 0, y: 0, zoom: 1 });
    const settled = vi.fn<(value: CanvasViewport) => void>();
    camera.observeSettled(settled);
    for (const x of [1, 2, 3, 4]) {
      camera.set({ x, y: 0, zoom: 1 });
      frames.run();
      expect(settled).not.toHaveBeenCalled();
    }
    frames.run();
    expect(settled).toHaveBeenCalledOnce();
  });

  it("reports settlement after a round trip, and stays silent without motion", () => {
    const camera = createCanvasCamera({ x: 0, y: 0, zoom: 1 });
    const settled = vi.fn<(value: CanvasViewport) => void>();
    camera.observeSettled(settled);
    camera.settle();
    expect(settled).not.toHaveBeenCalled();

    camera.set({ x: 40, y: 0, zoom: 1 });
    frames.run();
    camera.set({ x: 0, y: 0, zoom: 1 });
    camera.settle();
    expect(settled).toHaveBeenCalledExactlyOnceWith({ x: 0, y: 0, zoom: 1 });
    expect(frames.pending.size).toBe(0);
  });

  it("ignores a repeated camera value and releases its frames on dispose", () => {
    const camera = createCanvasCamera({ x: 5, y: 5, zoom: 2 });
    const projected = vi.fn<(value: CanvasViewport) => void>();
    const stop = camera.observeFrames(projected);
    camera.set({ x: 5, y: 5, zoom: 2 });
    expect(frames.pending.size).toBe(0);

    stop();
    camera.set({ x: 6, y: 5, zoom: 2 });
    frames.run();
    expect(projected).not.toHaveBeenCalled();

    camera.set({ x: 7, y: 5, zoom: 2 });
    camera.dispose();
    expect(frames.pending.size).toBe(0);
  });
});
