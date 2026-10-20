import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { effectScope, nextTick, shallowRef } from "vue";

import { useContentAdmission } from "../scene/use-content-admission";
import { admitVisibleIds, CONTENT_MOUNT_BUDGET } from "../scene/visibility-admission";
import { createCanvasCamera } from "../viewport/canvas-camera";
import { useTestAnimationFrames } from "./support/animation-frames";

function ids(count: number, offset = 0) {
  return new Set(Array.from({ length: count }, (_, index) => `node-${index + offset}`));
}

describe("visibility admission", () => {
  it("admits at most the budget, discards in full and reports a remaining backlog", () => {
    const admitted = admitVisibleIds(new Set(["kept", "dropped"]), ids(10), 3);

    expect(admitted).toEqual({ ids: ids(3), pending: true });
    expect(admitVisibleIds(ids(3), ids(3), 3)).toBeUndefined();
    expect(admitVisibleIds(ids(4), ids(3), 0)).toEqual({ ids: ids(3), pending: false });
  });
});

describe("content admission", () => {
  let frames: ReturnType<typeof useTestAnimationFrames>;

  beforeEach(() => (frames = useTestAnimationFrames()));
  afterEach(() => vi.unstubAllGlobals());

  it("spreads mounting across frames while the camera moves and flushes once it settles", async () => {
    const camera = createCanvasCamera({ x: 0, y: 0, zoom: 1 });
    const target = shallowRef<ReadonlySet<string>>(ids(2));
    const scope = effectScope();
    const admitted = scope.run(() => useContentAdmission(camera, target))!;
    try {
      expect(admitted.value).toEqual(ids(2));

      camera.set({ x: 100, y: 0, zoom: 1 });
      frames.run();
      target.value = ids(40);
      await nextTick();
      expect(admitted.value).toEqual(ids(2));

      for (const [index, x] of [200, 300].entries()) {
        camera.set({ x, y: 0, zoom: 1 });
        frames.run();
        expect(admitted.value.size).toBe(2 + CONTENT_MOUNT_BUDGET * (index + 1));
      }

      camera.settle();
      expect(admitted.value).toEqual(ids(40));
      expect(frames.pending.size).toBe(0);
    } finally {
      scope.stop();
    }
  });

  it("admits a settled camera immediately and stops draining when the scope ends", async () => {
    const camera = createCanvasCamera({ x: 0, y: 0, zoom: 1 });
    const target = shallowRef<ReadonlySet<string>>(new Set());
    const scope = effectScope();
    const admitted = scope.run(() => useContentAdmission(camera, target))!;

    target.value = ids(40);
    await nextTick();
    expect(admitted.value).toEqual(ids(40));
    expect(frames.pending.size).toBe(0);

    camera.set({ x: 10, y: 0, zoom: 1 });
    frames.run();
    target.value = ids(80);
    await nextTick();
    scope.stop();
    frames.run();
    expect(admitted.value).toEqual(ids(40));
  });
});
