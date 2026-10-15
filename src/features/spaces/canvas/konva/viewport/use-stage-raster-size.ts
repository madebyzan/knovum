import { useDebounceFn } from "@vueuse/core";
import { reactive, shallowRef, watch, type Reactive } from "vue";

const RASTER_STEP = 256;
const SHRINK_SETTLE_MS = 220;

function quantize(extent: number) {
  return Math.max(RASTER_STEP, Math.ceil(extent / RASTER_STEP) * RASTER_STEP);
}

/**
 * Raster size of the stage canvases.
 *
 * Every Konva layer reallocates and fully repaints its scene and hit canvas when the stage is resized, so a
 * host that animates - a workbench panel opening, a window resize - would pay that cost on each frame of the
 * animation. The raster therefore grows immediately, which keeps newly exposed area painted, and shrinks only
 * once the host size has settled. Quantizing both directions keeps ordinary layout jitter off the canvases.
 */
export function useStageRasterSize(host: Reactive<{ height: number; width: number }>) {
  const width = shallowRef(quantize(host.width));
  const height = shallowRef(quantize(host.height));
  const shrink = useDebounceFn(() => {
    width.value = quantize(host.width);
    height.value = quantize(host.height);
  }, SHRINK_SETTLE_MS);

  watch([() => host.width, () => host.height], ([nextWidth, nextHeight]) => {
    width.value = Math.max(width.value, quantize(nextWidth));
    height.value = Math.max(height.value, quantize(nextHeight));
    void shrink();
  });

  return reactive({ width, height });
}
