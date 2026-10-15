import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { effectScope, nextTick, reactive } from "vue";

import { useStageRasterSize } from "../konva/viewport/use-stage-raster-size";

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

it("grows the stage raster immediately and only shrinks once the host size settles", async () => {
  const host = reactive({ width: 1200, height: 800 });
  const scope = effectScope();
  const raster = scope.run(() => useStageRasterSize(host))!;
  expect({ ...raster }).toEqual({ width: 1280, height: 1024 });

  async function resizeHost(width: number) {
    host.width = width;
    await nextTick();
    return raster.width;
  }

  // A panel opening narrows the work area on every frame of its animation.
  expect(await resizeHost(1100)).toBe(1280);
  expect(await resizeHost(1000)).toBe(1280);
  expect(await resizeHost(900)).toBe(1280);
  expect(await resizeHost(1400)).toBe(1536);
  expect(await resizeHost(880)).toBe(1536);

  await vi.advanceTimersByTimeAsync(300);
  expect(raster.width).toBe(1024);
  scope.stop();
});
