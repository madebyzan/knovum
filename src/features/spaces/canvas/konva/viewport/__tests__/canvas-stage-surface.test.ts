import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { effectScope, nextTick, reactive } from "vue";

import { useStageSurfaceSize } from "../use-stage-surface-size";

beforeEach(() => vi.useFakeTimers());
afterEach(() => vi.useRealTimers());

it("grows the stage surface immediately and only shrinks once the window settles", async () => {
  const window = reactive({ width: 1200, height: 800 });
  const scope = effectScope();
  const surface = scope.run(() => useStageSurfaceSize(window))!;
  expect({ ...surface }).toEqual({ width: 1280, height: 1024 });

  async function resizeWindow(width: number) {
    window.width = width;
    await nextTick();
    return surface.width;
  }

  // A window drag passes through many sizes; each quantized step is reallocated at most once.
  expect(await resizeWindow(1100)).toBe(1280);
  expect(await resizeWindow(1000)).toBe(1280);
  expect(await resizeWindow(1400)).toBe(1536);
  expect(await resizeWindow(880)).toBe(1536);

  await vi.advanceTimersByTimeAsync(300);
  expect(surface.width).toBe(1024);
  scope.stop();
});
